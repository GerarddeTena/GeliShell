use super::{ConfigError, SelectorMode, ShellConfig};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{Write, stdout};

struct WizardOption {
    mode: SelectorMode,
    label: &'static str,
    description: &'static str,
}

const OPTIONS: &[WizardOption] = &[
    WizardOption {
        mode: SelectorMode::Always,
        label: "Always show selector",
        description: "pick with ↑↓ every time alternatives are available",
    },
    WizardOption {
        mode: SelectorMode::Auto,
        label: "Auto-execute first",
        description: "always run the native command without asking",
    },
    WizardOption {
        mode: SelectorMode::Once,
        label: "Ask once per command",
        description: "show selector once, remember for the session",
    },
];

pub fn run_first_run_wizard() -> Result<ShellConfig, ConfigError> {
    let mut stdout = stdout();
    terminal::enable_raw_mode().map_err(ConfigError::Read)?;

    // Oculta el cursor durante el wizard
    execute!(stdout, cursor::Hide).map_err(ConfigError::Read)?;

    let result = show_wizard(&mut stdout);

    execute!(stdout, cursor::Show).map_err(ConfigError::Read)?;
    terminal::disable_raw_mode().map_err(ConfigError::Read)?;
    result
}

fn show_wizard(stdout: &mut impl Write) -> Result<ShellConfig, ConfigError> {
    let mut selected = 0usize;

    // ── Guarda la posición inicial del cursor ─────────────────
    // Primer render desde la posición actual
    let start_row = cursor::position().map(|(_, row)| row).unwrap_or(0);

    // Render inicial
    render_wizard(stdout, selected, start_row)?;

    loop {
        if let Event::Key(key) = event::read().map_err(ConfigError::Read)? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            match key.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                        render_wizard(stdout, selected, start_row)?;
                    }
                }
                KeyCode::Down => {
                    if selected < OPTIONS.len() - 1 {
                        selected += 1;
                        render_wizard(stdout, selected, start_row)?;
                    }
                }
                KeyCode::Enter => {
                    clear_from(stdout, start_row, OPTIONS.len() + 8)?;
                    let mut config = ShellConfig::default();
                    config.behavior.selector_mode = OPTIONS[selected].mode.clone();
                    return Ok(config);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    clear_from(stdout, start_row, OPTIONS.len() + 8)?;
                    return Ok(ShellConfig::default());
                }
                _ => {}
            }
        }
    }
}

fn render_wizard(
    stdout: &mut impl Write,
    selected: usize,
    start_row: u16,
) -> Result<(), ConfigError> {
    // ── Siempre vuelve a la fila inicial — posición absoluta ──
    execute!(stdout, cursor::MoveTo(0, start_row),).map_err(ConfigError::Read)?;

    let width = 60usize;
    let border = "─".repeat(width - 2);

    // Header
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│{:^width$}│\r\n",
            " Welcome to GeliShell ",
            width = width - 2
        )),
        Print(format!("│{:^width$}│\r\n", " ", width = width - 2)),
        Print(format!(
            "│  {:<width$}│\r\n",
            "When alternatives exist, how should GeliShell choose?",
            width = width - 4
        )),
        Print(format!("│{:^width$}│\r\n", " ", width = width - 2)),
        ResetColor,
    )
    .map_err(ConfigError::Read)?;

    // Opciones
    for (i, opt) in OPTIONS.iter().enumerate() {
        let is_selected = i == selected;

        if is_selected {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print(format!(
                    "│  ❯ {:<width$}│\r\n",
                    format!("{:<22} {}", opt.label, opt.description),
                    width = width - 6
                )),
                ResetColor,
            )
            .map_err(ConfigError::Read)?;
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(format!(
                    "│    {:<width$}│\r\n",
                    format!("{:<22} {}", opt.label, opt.description),
                    width = width - 6
                )),
                ResetColor,
            )
            .map_err(ConfigError::Read)?;
        }
    }

    // Footer
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("│{:^width$}│\r\n", " ", width = width - 2)),
        Print(format!(
            "│  {:<width$}│\r\n",
            "↑↓ navigate  ·  Enter select  ·  Esc default",
            width = width - 4
        )),
        Print(format!("└{}┘\r\n", border)),
        ResetColor,
    )
    .map_err(ConfigError::Read)?;

    stdout.flush().map_err(ConfigError::Read)?;
    Ok(())
}

/// Limpia el modal volviendo a la fila de inicio
/// y borrando línea por línea hacia abajo
fn clear_from(stdout: &mut impl Write, start_row: u16, lines: usize) -> Result<(), ConfigError> {
    for i in 0..lines {
        execute!(
            stdout,
            cursor::MoveTo(0, start_row + i as u16),
            terminal::Clear(ClearType::CurrentLine),
        )
        .map_err(ConfigError::Read)?;
    }
    // Deja el cursor en la fila de inicio limpia
    execute!(stdout, cursor::MoveTo(0, start_row),).map_err(ConfigError::Read)?;
    stdout.flush().map_err(ConfigError::Read)?;
    Ok(())
}
