use crate::shell::assistant::params::{AssistantParameter, filter_parameters};
use crate::shell::assistant::qwen::BootstrapEvent;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write, stdout};
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantMenuSelection {
    Closed,
    Selected {
        parameter: AssistantParameter,
        filter: String,
    },
}

pub fn show_assistant_menu() -> io::Result<AssistantMenuSelection> {
    let mut out = stdout();
    terminal::enable_raw_mode()?;
    execute!(
        out,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Hide,
    )?;

    let result = run_assistant_menu(&mut out);

    let cleanup_screen = execute!(
        out,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show,
        terminal::LeaveAlternateScreen,
    );
    let cleanup_raw = terminal::disable_raw_mode();

    cleanup_screen?;
    cleanup_raw?;
    result
}

pub fn show_how_to_confirmation_panel(explanation: &str, command: &str) -> io::Result<bool> {
    let mut out = stdout();
    terminal::enable_raw_mode()?;
    execute!(
        out,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Hide,
    )?;

    let result = run_how_to_confirmation_panel(&mut out, explanation, command);

    let cleanup_screen = execute!(
        out,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show,
        terminal::LeaveAlternateScreen,
    );
    let cleanup_raw = terminal::disable_raw_mode();

    cleanup_screen?;
    cleanup_raw?;
    result
}

pub async fn show_model_bootstrap_progress(
    mut rx: mpsc::UnboundedReceiver<BootstrapEvent>,
) -> io::Result<()> {
    let mut out = stdout();
    terminal::enable_raw_mode()?;
    execute!(
        out,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Hide,
    )?;

    let mut state = BootstrapState::default();

    let loop_result = run_bootstrap_loop(&mut out, &mut rx, &mut state).await;

    // Cleanup garantizado — se ejecuta independientemente del resultado
    // del loop. El orden importa: primero disable_raw_mode, luego
    // LeaveAlternateScreen para que conpty restaure el modo correcto.
    let _ = terminal::disable_raw_mode();
    let _ = execute!(
        out,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show,
        terminal::LeaveAlternateScreen,
    );

    loop_result
}

async fn run_bootstrap_loop(
    out: &mut impl Write,
    rx: &mut mpsc::UnboundedReceiver<BootstrapEvent>,
    state: &mut BootstrapState,
) -> io::Result<()> {
    loop {
        while let Ok(event) = rx.try_recv() {
            state.apply(event);
        }

        render_bootstrap_frame(out, state)?;

        if state.done {
            break;
        }

        if event::poll(std::time::Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                let ctrl_c =
                    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
                let escaped = matches!(key.code, KeyCode::Esc);
                if ctrl_c || escaped {
                    state.status = "bootstrap interrupted by user".to_owned();
                    state.done = true;
                }
            }
        }

        if rx.is_closed() && !state.done {
            if state.status.is_empty() {
                state.status = "model bootstrap channel closed".to_owned();
            }
            state.done = true;
        }

        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
    }

    Ok(())
}

fn run_how_to_confirmation_panel(
    out: &mut impl Write,
    explanation: &str,
    command: &str,
) -> io::Result<bool> {
    render_how_to_confirmation_panel(out, explanation, command)?;

    loop {
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }

        let ctrl_c =
            key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return Ok(false),
            KeyCode::Enter => return Ok(false),
            _ if ctrl_c => return Ok(false),
            _ => {}
        }
    }
}

fn render_how_to_confirmation_panel(
    out: &mut impl Write,
    explanation: &str,
    command: &str,
) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let width = 98usize;
    let border = "─".repeat(width - 2);

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$}│\r\n",
            "GeliShell Assistant --how-to",
            width = width - 3
        )),
        Print(format!("├{}┤\r\n", border)),
        ResetColor,
    )?;

    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(format!(
            "│ {:<width$}│\r\n",
            "Explicación:",
            width = width - 3
        )),
        ResetColor,
    )?;

    for line in wrap_lines(explanation, width - 6) {
        execute!(
            out,
            SetForegroundColor(Color::Yellow),
            Print(format!("│  {:<width$}│\r\n", line, width = width - 6)),
            ResetColor,
        )?;
    }

    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("│ {:<width$}│\r\n", " ", width = width - 3)),
        Print(format!("│ {:<width$}│\r\n", "Comando:", width = width - 3)),
        ResetColor,
    )?;

    for line in wrap_lines(command, width - 6) {
        execute!(
            out,
            SetForegroundColor(Color::Green),
            Print(format!("│  {:<width$}│\r\n", line, width = width - 6)),
            ResetColor,
        )?;
    }

    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("│ {:<width$}│\r\n", " ", width = width - 3)),
        Print(format!(
            "│ {:<width$}│\r\n",
            "¿Deseas ejecutarlo? [y/n]",
            width = width - 3
        )),
        Print(format!(
            "│ {:<width$}│\r\n",
            "y = ejecutar · n/Esc/Enter/Ctrl+C = cancelar",
            width = width - 3
        )),
        SetForegroundColor(Color::Cyan),
        Print(format!("└{}┘\r\n", border)),
        ResetColor,
    )?;

    out.flush()?;
    Ok(())
}

pub fn show_assistant_error_panel(message: &str) -> io::Result<()> {
    let mut out = stdout();
    terminal::enable_raw_mode()?;
    execute!(
        out,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Hide,
    )?;

    let result = run_assistant_error_panel(&mut out, message);

    let cleanup_screen = execute!(
        out,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show,
        terminal::LeaveAlternateScreen,
    );
    let cleanup_raw = terminal::disable_raw_mode();

    cleanup_screen?;
    cleanup_raw?;
    result
}

fn run_assistant_error_panel(out: &mut impl Write, message: &str) -> io::Result<()> {
    render_assistant_error_panel(out, message)?;
    loop {
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }

        let ctrl_c =
            key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl_c || key.code == KeyCode::Esc || key.code == KeyCode::Enter {
            return Ok(());
        }
    }
}

fn render_assistant_error_panel(out: &mut impl Write, message: &str) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let width = 96usize;
    let border = "─".repeat(width - 2);

    execute!(
        out,
        SetForegroundColor(Color::Red),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$}│\r\n",
            "GeliShell Assistant Error",
            width = width - 3
        )),
        Print(format!("├{}┤\r\n", border)),
        ResetColor,
    )?;

    for line in wrap_lines(message, width - 6) {
        execute!(
            out,
            SetForegroundColor(Color::Yellow),
            Print(format!("│  {:<width$}│\r\n", line, width = width - 6)),
            ResetColor,
        )?;
    }

    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("│ {:<width$}│\r\n", " ", width = width - 3)),
        Print(format!(
            "│ {:<width$}│\r\n",
            "Press Enter/Esc/Ctrl+C to close",
            width = width - 3
        )),
        SetForegroundColor(Color::Red),
        Print(format!("└{}┘\r\n", border)),
        ResetColor,
    )?;

    out.flush()?;
    Ok(())
}

fn wrap_lines(input: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in input.split_whitespace() {
        let projected = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };
        if projected > max_width && !current.is_empty() {
            lines.push(current.clone());
            current.clear();
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn run_assistant_menu(out: &mut impl Write) -> io::Result<AssistantMenuSelection> {
    let mut filter = String::new();
    let mut selected = 0usize;

    render_assistant_menu(out, &filter, selected)?;

    loop {
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }

        let ctrl_c =
            key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl_c || key.code == KeyCode::Esc {
            return Ok(AssistantMenuSelection::Closed);
        }

        match key.code {
            KeyCode::Up => {
                selected = selected.saturating_sub(1);
            }
            KeyCode::Down => {
                let max = filter_parameters(&filter).len().saturating_sub(1);
                if selected < max {
                    selected += 1;
                }
            }
            KeyCode::Backspace => {
                filter.pop();
                selected = 0;
            }
            KeyCode::Char(ch) => {
                filter.push(ch);
                selected = 0;
            }
            KeyCode::Enter => {
                let filtered = filter_parameters(&filter);
                if filtered.is_empty() {
                    continue;
                }
                let clamped = selected.min(filtered.len() - 1);
                return Ok(AssistantMenuSelection::Selected {
                    parameter: filtered[clamped],
                    filter,
                });
            }
            _ => {}
        }

        let max = filter_parameters(&filter).len().saturating_sub(1);
        selected = selected.min(max);
        render_assistant_menu(out, &filter, selected)?;
    }
}

fn render_assistant_menu(out: &mut impl Write, filter: &str, selected: usize) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let options = filter_parameters(filter);
    let width = 84usize;
    let border = "─".repeat(width - 2);

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$}│\r\n",
            "GeliShell Assistant",
            width = width - 3
        )),
        Print(format!(
            "│ {:<width$}│\r\n",
            "↑↓ navigate · Enter select · Esc/Ctrl+C close",
            width = width - 3
        )),
        Print(format!("├{}┤\r\n", border)),
        ResetColor,
    )?;

    if options.is_empty() {
        execute!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "│ {:<width$}│\r\n",
                "No matching actions. Keep typing to refine.",
                width = width - 3
            )),
            ResetColor,
        )?;
    } else {
        for (idx, option) in options.iter().enumerate() {
            let text = format!("{:<22} {}", option.label, option.description);
            if idx == selected {
                execute!(
                    out,
                    SetBackgroundColor(Color::DarkBlue),
                    SetForegroundColor(Color::White),
                    Print(format!("│  ❯ {:<width$}│\r\n", text, width = width - 6)),
                    ResetColor,
                )?;
            } else {
                execute!(
                    out,
                    SetForegroundColor(Color::Magenta),
                    Print(format!("│    {:<width$}│\r\n", text, width = width - 6)),
                    ResetColor,
                )?;
            }
        }
    }

    let filter_label = if filter.is_empty() {
        "[ Type to filter... ]".to_owned()
    } else {
        format!("[ Type to filter... ] {filter}")
    };

    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("│ {:<width$}│\r\n", " ", width = width - 3)),
        SetForegroundColor(Color::Yellow),
        Print(format!(
            "│ {:<width$}│\r\n",
            filter_label,
            width = width - 3
        )),
        SetForegroundColor(Color::Cyan),
        Print(format!("└{}┘\r\n", border)),
        ResetColor,
    )?;
    out.flush()?;
    Ok(())
}

#[derive(Debug, Default)]
struct BootstrapState {
    status: String,
    model_path: String,
    downloaded: u64,
    total: Option<u64>,
    loaded_size: u64,
    downloaded_file: bool,
    done: bool,
}

impl BootstrapState {
    fn apply(&mut self, event: BootstrapEvent) {
        match event {
            BootstrapEvent::CheckingModel { path } => {
                self.model_path = path;
                self.status = "checking model availability...".to_owned();
            }
            BootstrapEvent::ExistingModelFound { path, size_bytes } => {
                self.model_path = path;
                self.loaded_size = size_bytes;
                self.downloaded_file = false;
                self.status = "model already cached locally".to_owned();
            }
            BootstrapEvent::Downloading {
                downloaded_bytes,
                total_bytes,
            } => {
                self.downloaded = downloaded_bytes;
                self.total = total_bytes;
                self.downloaded_file = true;
                self.status = "downloading qwen gguf model...".to_owned();
            }
            BootstrapEvent::VerifyingModel => {
                self.status = "verifying gguf integrity...".to_owned();
            }
            BootstrapEvent::ModelLoaded { path, size_bytes } => {
                self.model_path = path;
                self.loaded_size = size_bytes;
                self.status = if self.downloaded_file {
                    "download complete — model loaded".to_owned()
                } else {
                    "model loaded from local cache".to_owned()
                };
                self.done = true;
            }
            BootstrapEvent::Failed { reason } => {
                self.status = format!("bootstrap failed: {reason}");
                self.done = true;
            }
        }
    }
}

fn render_bootstrap_frame(out: &mut impl Write, state: &BootstrapState) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let width = 84usize;
    let border = "─".repeat(width - 2);
    let bar = build_progress_bar(state.downloaded, state.total, 48);

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$}│\r\n",
            "GeliShell Assistant bootstrap",
            width = width - 3
        )),
        Print(format!(
            "│ {:<width$}│\r\n",
            "Preparing local Qwen model in ~/.config/geliShell/models/",
            width = width - 3
        )),
        Print(format!("├{}┤\r\n", border)),
        ResetColor,
        SetForegroundColor(Color::Magenta),
        Print(format!(
            "│ {:<width$}│\r\n",
            state.status,
            width = width - 3
        )),
        SetForegroundColor(Color::Yellow),
        Print(format!("│ {:<width$}│\r\n", bar, width = width - 3)),
        SetForegroundColor(Color::DarkGrey),
        Print(format!(
            "│ {:<width$}│\r\n",
            format!("model path: {}", state.model_path),
            width = width - 3
        )),
        Print(format!(
            "│ {:<width$}│\r\n",
            format!(
                "model size: {} bytes",
                state.loaded_size.max(state.downloaded)
            ),
            width = width - 3
        )),
        Print(format!(
            "│ {:<width$}│\r\n",
            "Esc/Ctrl+C closes this panel only",
            width = width - 3
        )),
        SetForegroundColor(Color::Cyan),
        Print(format!("└{}┘\r\n", border)),
        ResetColor,
    )?;

    out.flush()?;
    Ok(())
}

fn build_progress_bar(downloaded: u64, total: Option<u64>, width: usize) -> String {
    let Some(total) = total else {
        return format!("[{}] {} bytes", "▮".repeat(width.min(8)), downloaded);
    };

    if total == 0 {
        return "[------------------------------------------------] 0%".to_owned();
    }

    let ratio = (downloaded as f64 / total as f64).clamp(0.0, 1.0);
    let filled = ((ratio * width as f64).round() as usize).min(width);
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}] {:>3}%",
        "█".repeat(filled),
        "·".repeat(empty),
        (ratio * 100.0).round() as u8
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_handles_unknown_total() {
        let bar = build_progress_bar(1024, None, 24);
        assert!(bar.contains("bytes"));
    }

    #[test]
    fn progress_bar_handles_known_total() {
        let bar = build_progress_bar(50, Some(100), 20);
        assert!(bar.contains("50%"));
    }
}
