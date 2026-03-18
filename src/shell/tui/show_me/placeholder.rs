use super::error::ShowMeError;
use crossterm::{cursor, execute, terminal};
use std::io::{self, BufRead, Write};

pub(crate) fn resolve_placeholders(
    command: &str,
    stdout: &mut impl Write,
) -> Result<String, ShowMeError> {
    let markers = extract_markers(command);
    if markers.is_empty() {
        return Ok(command.to_owned());
    }

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let values = prompt_for_marker_values(&markers, stdout, &mut stdin_lock)?;
    Ok(apply_marker_values(command, &markers, &values))
}

fn prompt_for_marker_values(
    markers: &[String],
    stdout: &mut impl Write,
    input: &mut impl BufRead,
) -> Result<Vec<String>, ShowMeError> {
    let mut values = Vec::with_capacity(markers.len());

    for marker in markers {
        terminal::disable_raw_mode().map_err(terminal_error)?;
        execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen).map_err(terminal_error)?;

        let read_result = read_marker_value(marker, stdout, input);

        let resume_screen = execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::MoveTo(0, 0),
            cursor::Hide
        )
        .map_err(terminal_error);
        let resume_raw = terminal::enable_raw_mode().map_err(terminal_error);

        resume_screen?;
        resume_raw?;
        values.push(read_result?);
    }

    Ok(values)
}

fn read_marker_value(
    marker: &str,
    stdout: &mut impl Write,
    input: &mut impl BufRead,
) -> Result<String, ShowMeError> {
    write!(stdout, "Enter <{marker}>: ")?;
    stdout.flush()?;

    let mut line = String::new();
    input.read_line(&mut line)?;
    Ok(line.trim_end_matches(&['\r', '\n'][..]).to_owned())
}

fn extract_markers(command: &str) -> Vec<String> {
    let mut markers = Vec::new();
    let mut remaining = command;

    while let Some(start) = remaining.find('<') {
        let after_start = &remaining[start + 1..];
        let Some(end) = after_start.find('>') else {
            break;
        };

        let marker = &after_start[..end];
        if !marker.is_empty() {
            markers.push(marker.to_owned());
        }

        remaining = &after_start[end + 1..];
    }

    markers
}

fn apply_marker_values(command: &str, markers: &[String], values: &[String]) -> String {
    let mut resolved = command.to_owned();

    for (marker, value) in markers.iter().zip(values) {
        let token = format!("<{marker}>");
        resolved = resolved.replacen(&token, value, 1);
    }

    resolved
}

fn terminal_error(error: io::Error) -> ShowMeError {
    ShowMeError::Terminal(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_placeholders_replaces_single_marker() {
        let command = "cat <archivo>";
        let markers = extract_markers(command);
        let values = vec!["notas.md".to_owned()];

        let resolved = apply_marker_values(command, &markers, &values);

        assert_eq!(resolved, "cat notas.md");
    }

    #[test]
    fn resolve_placeholders_returns_original_when_no_markers() {
        let mut output = Vec::new();
        let resolved = resolve_placeholders("ls -la", &mut output).unwrap();

        assert_eq!(resolved, "ls -la");
    }
}
