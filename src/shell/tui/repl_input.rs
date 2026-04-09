use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplInputAction {
    Command(String),
    Exit,
    OpenHelp,
    OpenConfig,
    OpenAssistant,
    Clear,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialCommand {
    Stop,
    Search,
}

struct RawModeGuard;

impl RawModeGuard {
    fn acquire() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

/// Drena eventos stale del buffer de input de crossterm mientras raw mode está activo.
/// Previene que un ENTER buffereado durante la ejecución de un comando sea
/// procesado como entrada en el siguiente ciclo del REPL (bug del doble ENTER).
fn flush_input_buffer() {
    use crossterm::event::{poll, read};
    use std::time::Duration;
    while matches!(poll(Duration::from_millis(0)), Ok(true)) {
        let _ = read();
    }
}

pub fn read_repl_input(
    prompt: &str,
    history: &[String],
    completion_pool: &[String],
    g_jump_paths: &[String],
    ghost_color: u8,
) -> io::Result<ReplInputAction> {
    let mut stdout = io::stdout();
    let _raw_mode = RawModeGuard::acquire()?;

    // Drenar cualquier evento stale (ENTER de la ejecución anterior)
    // mientras raw mode está activo, para evitar el bug del doble ENTER.
    flush_input_buffer();

    let mut input = String::new();
    let mut cursor_pos = 0usize;
    let mut history_cursor: Option<usize> = None;
    let mut history_draft = String::new();

    render_input_line(
        &mut stdout,
        prompt,
        &input,
        cursor_pos,
        history,
        completion_pool,
        g_jump_paths,
        ghost_color,
    )?;

    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                if let Some(shortcut) = shortcut_action(&key) {
                    commit_current_line(&mut stdout, prompt, &input)?;
                    return Ok(shortcut);
                }

                match key.code {
                    KeyCode::Enter => {
                        let command = input.trim().to_owned();
                        commit_current_line(&mut stdout, prompt, &input)?;
                        return Ok(ReplInputAction::Command(command));
                    }
                    KeyCode::Backspace => {
                        if cursor_pos > 0 {
                            remove_char_before(&mut input, &mut cursor_pos);
                            history_cursor = None;
                        }
                    }
                    KeyCode::Delete => {
                        if cursor_pos < char_len(&input) {
                            remove_char_at(&mut input, cursor_pos);
                            history_cursor = None;
                        }
                    }
                    KeyCode::Left => {
                        cursor_pos = cursor_pos.saturating_sub(1);
                    }
                    KeyCode::Right => {
                        if let Some(suffix) = find_suggestion_suffix(
                            &input,
                            cursor_pos,
                            history,
                            completion_pool,
                            g_jump_paths,
                        ) {
                            insert_str_at_cursor(&mut input, cursor_pos, &suffix);
                            cursor_pos += suffix.chars().count();
                            history_cursor = None;
                        } else if cursor_pos < char_len(&input) {
                            cursor_pos += 1;
                        }
                    }
                    KeyCode::Home => {
                        cursor_pos = 0;
                    }
                    KeyCode::End => {
                        cursor_pos = char_len(&input);
                    }
                    KeyCode::Up => {
                        history_up(
                            history,
                            &mut input,
                            &mut cursor_pos,
                            &mut history_cursor,
                            &mut history_draft,
                        );
                    }
                    KeyCode::Down => {
                        history_down(
                            history,
                            &mut input,
                            &mut cursor_pos,
                            &mut history_cursor,
                            &history_draft,
                        );
                    }
                    KeyCode::Tab => {
                        if let Some(suffix) = find_suggestion_suffix(
                            &input,
                            cursor_pos,
                            history,
                            completion_pool,
                            g_jump_paths,
                        ) {
                            insert_str_at_cursor(&mut input, cursor_pos, &suffix);
                            cursor_pos += suffix.chars().count();
                            history_cursor = None;
                        }
                    }
                    KeyCode::Char(ch) => {
                        insert_char_at_cursor(&mut input, cursor_pos, ch);
                        cursor_pos += 1;
                        history_cursor = None;
                    }
                    _ => {}
                }

                render_input_line(
                    &mut stdout,
                    prompt,
                    &input,
                    cursor_pos,
                    history,
                    completion_pool,
                    g_jump_paths,
                    ghost_color,
                )?;
            }
            Event::Paste(paste) => {
                insert_str_at_cursor(&mut input, cursor_pos, &paste);
                cursor_pos += paste.chars().count();
                history_cursor = None;

                render_input_line(
                    &mut stdout,
                    prompt,
                    &input,
                    cursor_pos,
                    history,
                    completion_pool,
                    g_jump_paths,
                    ghost_color,
                )?;
            }
            _ => {}
        }
    }
}

pub fn parse_special_command(input: &str) -> Option<SpecialCommand> {
    match input.trim() {
        ":stop" | ":stop*" => Some(SpecialCommand::Stop),
        ":search" | ":search*" => Some(SpecialCommand::Search),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn render_input_line(
    stdout: &mut impl Write,
    prompt: &str,
    input: &str,
    cursor_pos: usize,
    history: &[String],
    completion_pool: &[String],
    g_jump_paths: &[String],
    ghost_color: u8,
) -> io::Result<()> {
    let suggestion =
        find_suggestion_suffix(input, cursor_pos, history, completion_pool, g_jump_paths);

    execute!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
        Print(prompt),
        Print(input),
    )?;

    let mut ghost_len = 0usize;
    if let Some(suffix) = suggestion {
        ghost_len = suffix.chars().count();
        execute!(
            stdout,
            SetForegroundColor(Color::AnsiValue(ghost_color)),
            Print(suffix),
            ResetColor,
        )?;
    }

    let tail_len = ghost_len + char_len(input).saturating_sub(cursor_pos);
    if tail_len > 0 {
        execute!(
            stdout,
            cursor::MoveLeft(tail_len.min(u16::MAX as usize) as u16)
        )?;
    }

    stdout.flush()?;
    Ok(())
}

fn commit_current_line(stdout: &mut impl Write, prompt: &str, input: &str) -> io::Result<()> {
    execute!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
        Print(prompt),
        Print(input),
        Print("\r\n"),
    )?;
    stdout.flush()?;
    Ok(())
}

fn shortcut_action(key: &KeyEvent) -> Option<ReplInputAction> {
    let modifiers = key.modifiers;

    match key.code {
        KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
            Some(ReplInputAction::Exit)
        }
        KeyCode::Char('h') if modifiers.contains(KeyModifiers::CONTROL) => {
            Some(ReplInputAction::OpenHelp)
        }
        KeyCode::Char('?') if modifiers.contains(KeyModifiers::CONTROL) => {
            Some(ReplInputAction::OpenHelp)
        }
        KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
            Some(ReplInputAction::Clear)
        }
        KeyCode::Char('s')
            if modifiers.contains(KeyModifiers::CONTROL)
                && modifiers.contains(KeyModifiers::ALT) =>
        {
            Some(ReplInputAction::OpenConfig)
        }
        KeyCode::Char('g')
            if modifiers.contains(KeyModifiers::CONTROL)
                && modifiers.contains(KeyModifiers::ALT) =>
        {
            Some(ReplInputAction::OpenAssistant)
        }
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            Some(ReplInputAction::Search)
        }
        _ => None,
    }
}

fn history_up(
    history: &[String],
    input: &mut String,
    cursor_pos: &mut usize,
    history_cursor: &mut Option<usize>,
    history_draft: &mut String,
) {
    if history.is_empty() {
        return;
    }

    match history_cursor {
        Some(index) => {
            if *index > 0 {
                *index -= 1;
            }
        }
        None => {
            *history_draft = input.clone();
            *history_cursor = Some(history.len() - 1);
        }
    }

    if let Some(index) = *history_cursor {
        *input = history[index].clone();
        *cursor_pos = char_len(input);
    }
}

fn history_down(
    history: &[String],
    input: &mut String,
    cursor_pos: &mut usize,
    history_cursor: &mut Option<usize>,
    history_draft: &str,
) {
    if history.is_empty() {
        return;
    }

    match history_cursor {
        Some(index) if *index + 1 < history.len() => {
            *index += 1;
            *input = history[*index].clone();
        }
        Some(_) => {
            *history_cursor = None;
            *input = history_draft.to_owned();
        }
        None => return,
    }

    *cursor_pos = char_len(input);
}

fn find_suggestion_suffix(
    input: &str,
    cursor_pos: usize,
    history: &[String],
    completion_pool: &[String],
    g_jump_paths: &[String],
) -> Option<String> {
    if cursor_pos != char_len(input) || input.is_empty() {
        return None;
    }

    for candidate in history.iter().rev() {
        if candidate.starts_with(input) && candidate.len() > input.len() {
            return Some(candidate[input.len()..].to_owned());
        }
    }

    if input.starts_with("g ") {
        for path in g_jump_paths {
            let candidate = format!("g {path}");
            if candidate.starts_with(input) && candidate.len() > input.len() {
                return Some(candidate[input.len()..].to_owned());
            }
        }
    }

    if input.contains(' ') {
        return None;
    }

    for candidate in completion_pool {
        if candidate.starts_with(input) && candidate.len() > input.len() {
            return Some(candidate[input.len()..].to_owned());
        }
    }

    None
}

fn char_len(input: &str) -> usize {
    input.chars().count()
}

fn insert_char_at_cursor(input: &mut String, cursor_pos: usize, ch: char) {
    let byte = byte_index(input, cursor_pos);
    input.insert(byte, ch);
}

fn insert_str_at_cursor(input: &mut String, cursor_pos: usize, value: &str) {
    let byte = byte_index(input, cursor_pos);
    input.insert_str(byte, value);
}

fn remove_char_before(input: &mut String, cursor_pos: &mut usize) {
    if *cursor_pos == 0 {
        return;
    }
    let remove_at = *cursor_pos - 1;
    remove_char_at(input, remove_at);
    *cursor_pos = remove_at;
}

fn remove_char_at(input: &mut String, char_pos: usize) {
    let start = byte_index(input, char_pos);
    let end = byte_index(input, char_pos + 1);
    if start < end && end <= input.len() {
        input.replace_range(start..end, "");
    }
}

fn byte_index(input: &str, char_pos: usize) -> usize {
    if char_pos == 0 {
        return 0;
    }

    input
        .char_indices()
        .nth(char_pos)
        .map(|(idx, _)| idx)
        .unwrap_or(input.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stop_special_command() {
        assert_eq!(parse_special_command(":stop"), Some(SpecialCommand::Stop));
        assert_eq!(parse_special_command(":stop*"), Some(SpecialCommand::Stop));
    }

    #[test]
    fn parses_search_special_command() {
        assert_eq!(
            parse_special_command(":search"),
            Some(SpecialCommand::Search)
        );
        assert_eq!(
            parse_special_command(":search*"),
            Some(SpecialCommand::Search)
        );
    }

    #[test]
    fn returns_none_for_regular_command() {
        assert_eq!(parse_special_command("ls"), None);
        assert_eq!(parse_special_command("geli-helpme"), None);
    }

    #[test]
    fn suggestions_prioritize_history_over_commands() {
        let history = vec!["git status --short".to_owned()];
        let commands = vec!["git".to_owned(), "grep".to_owned()];
        let suggestion = find_suggestion_suffix("git s", 5, &history, &commands, &[]);
        assert_eq!(suggestion, Some("tatus --short".to_owned()));
    }

    #[test]
    fn suggestions_fallback_to_command_pool() {
        let history = vec!["ls -la".to_owned()];
        let commands = vec!["clear".to_owned(), "cls".to_owned()];
        let suggestion = find_suggestion_suffix("cl", 2, &history, &commands, &[]);
        assert_eq!(suggestion, Some("ear".to_owned()));
    }

    #[test]
    fn suggestions_include_g_jump_paths() {
        let history: Vec<String> = Vec::new();
        let commands = vec!["g".to_owned()];
        let g_paths = vec!["C:/Users/Gerard/RustroverProjects/GeliShell".to_owned()];
        let input = "g C:/Users/Gerard/Rust";
        let suggestion =
            find_suggestion_suffix(input, input.chars().count(), &history, &commands, &g_paths);
        assert_eq!(suggestion, Some("roverProjects/GeliShell".to_owned()));
    }

    #[test]
    fn ctrl_alt_g_shortcut_opens_assistant() {
        let key = KeyEvent::new(
            KeyCode::Char('g'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        assert_eq!(shortcut_action(&key), Some(ReplInputAction::OpenAssistant));
    }
}
