use super::{CommandSelector, SelectionResult};
use crate::shell::translator::resolver::ResolvedCommand;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{Write, stdout};

pub struct ModalSelector;

impl ModalSelector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ModalSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandSelector for ModalSelector {
    fn select(&self, resolved: &ResolvedCommand) -> SelectionResult {
        // Construye la lista de opciones — nativo primero
        let options = resolved.all_options();
        if options.is_empty() {
            return SelectionResult::Selected(resolved.preferred.clone());
        }

        // Si solo hay una opción — no hace falta selector
        if options.len() == 1 {
            return SelectionResult::Selected(options[0].to_owned());
        }

        let mut stdout = stdout();
        terminal::enable_raw_mode().ok();

        let result = show_modal(&mut stdout, &options);

        terminal::disable_raw_mode().ok();
        result
    }
}

fn show_modal(stdout: &mut impl Write, options: &[&str]) -> SelectionResult {
    let mut selected = 0usize;
    let total = options.len();

    // Guarda la posición del cursor antes del primer render.
    // RestorePosition se usa en lugar de MoveUp para volver al inicio del modal.
    execute!(stdout, cursor::SavePosition).ok();

    loop {
        render_modal(stdout, options, selected).ok();

        match event::read() {
            Ok(Event::Key(key)) => {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                match key.code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < total - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        let chosen = options[selected].to_owned();
                        clear_modal(stdout, total + 5).ok();
                        return SelectionResult::Selected(chosen);
                    }
                    KeyCode::Esc => {
                        clear_modal(stdout, total + 5).ok();
                        return SelectionResult::Cancelled;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn render_modal(stdout: &mut impl Write, options: &[&str], selected: usize) -> std::io::Result<()> {
    let width = 52usize;
    let border = "─".repeat(width - 2);

    execute!(stdout, cursor::MoveToColumn(0))?;

    // Header
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│  {:<width$}│\r\n",
            "GeliShell — choose command",
            width = width - 4
        )),
        Print(format!("│{:^width$}│\r\n", " ", width = width - 2)),
        ResetColor,
    )?;

    // Opciones
    for (i, opt) in options.iter().enumerate() {
        let is_selected = i == selected;
        let prefix = if i == 0 { "native" } else { "" };

        if is_selected {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print(format!(
                    "│  ❯ {:<28} {:>10}  │\r\n",
                    truncate(opt, 28),
                    prefix
                )),
                ResetColor,
            )?;
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(format!(
                    "│    {:<28} {:>10}  │\r\n",
                    truncate(opt, 28),
                    prefix
                )),
                ResetColor,
            )?;
        }
    }

    // Footer
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("│{:^width$}│\r\n", " ", width = width - 2)),
        Print(format!(
            "│  {:<width$}│\r\n",
            "↑↓ navigate  ·  Enter execute  ·  Esc cancel",
            width = width - 4
        )),
        Print(format!("└{}┘\r\n", border)),
        ResetColor,
    )?;

    stdout.flush()?;

    // Vuelve al inicio del modal para el siguiente render usando posición guardada.
    // Nunca usar cursor::MoveUp — causa artefactos en terminales sin soporte completo ANSI.
    execute!(stdout, cursor::RestorePosition)?;
    Ok(())
}

fn clear_modal(stdout: &mut impl Write, lines: usize) -> std::io::Result<()> {
    // Vuelve al inicio del modal antes de borrar
    execute!(stdout, cursor::RestorePosition)?;
    for _ in 0..lines {
        execute!(
            stdout,
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveDown(1),
        )?;
    }
    // Vuelve al inicio tras limpiar para dejar el cursor en la posición correcta
    execute!(stdout, cursor::RestorePosition)?;
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    // Comparar char count, no byte count, para evitar pánico en caracteres multibyte UTF-8
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}
