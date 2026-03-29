use crate::t;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write, stdout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpMenuAction {
    None,
    Exit,
    Clear,
    Stop,
    Search,
}

struct HelpRow {
    shortcut: &'static str,
    command: &'static str,
    description: String,
    action: HelpMenuAction,
}

const COL_SHORTCUT: usize = 12;
const COL_COMMAND: usize = 14;
const COL_DESCRIPTION: usize = 66;

/// Fila Y (0-based) donde empiezan las filas de datos en la pantalla.
/// Estructura: borde(0) título(1) vacío(2) cabecera(3) separador(4) datos(5…)
const HELP_DATA_START_ROW: u16 = 5;

fn help_rows() -> Vec<HelpRow> {
    vec![
        HelpRow {
            shortcut: "^C",
            command: ":stop*",
            description: t!("tui.help.stop_desc"),
            action: HelpMenuAction::Stop,
        },
        HelpRow {
            shortcut: "^D",
            command: "exit",
            description: t!("tui.help.exit_desc"),
            action: HelpMenuAction::Exit,
        },
        HelpRow {
            shortcut: "^L",
            command: "clear",
            description: t!("tui.help.clear_desc"),
            action: HelpMenuAction::Clear,
        },
        HelpRow {
            shortcut: "^S",
            command: ":search*",
            description: t!("tui.help.search_desc"),
            action: HelpMenuAction::Search,
        },
    ]
}

pub fn show_help_menu() -> io::Result<HelpMenuAction> {
    let mut out = stdout();
    terminal::enable_raw_mode()?;
    execute!(
        out,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Hide,
    )?;

    let result = run_help_menu(&mut out);

    let screen_cleanup = execute!(
        out,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show,
        terminal::LeaveAlternateScreen,
    );
    let raw_cleanup = terminal::disable_raw_mode();

    screen_cleanup?;
    raw_cleanup?;
    result
}

fn run_help_menu(out: &mut impl Write) -> io::Result<HelpMenuAction> {
    let mut row = 0usize;
    let mut col = 0usize;
    let rows = help_rows();

    // Render inicial completo — con Clear(All) para establecer la pantalla
    render_help_menu(out, row, col, &rows)?;

    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                let prev_row = row;
                let prev_col = col;

                match key.code {
                    KeyCode::Up => {
                        if row > 0 {
                            row -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if row + 1 < rows.len() {
                            row += 1;
                        }
                    }
                    KeyCode::Left => {
                        if col > 0 {
                            col -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if col < 2 {
                            col += 1;
                        }
                    }
                    KeyCode::Enter => {
                        return Ok(rows[row].action);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        return Ok(HelpMenuAction::None);
                    }
                    _ => {}
                }

                // Render diferencial: solo actualiza las filas que cambiaron
                if row != prev_row {
                    update_help_row(out, prev_row, &rows[prev_row], false, col)?;
                    update_help_row(out, row, &rows[row], true, col)?;
                } else if col != prev_col {
                    update_help_row(out, row, &rows[row], true, col)?;
                }
            }
            _ => {}
        }
    }
}

/// Actualiza una única fila de datos en su posición Y exacta.
/// Evita el Clear(All) completo — elimina el flicker en navegación.
fn update_help_row(
    out: &mut impl Write,
    row_idx: usize,
    item: &HelpRow,
    selected: bool,
    col: usize,
) -> io::Result<()> {
    let y = HELP_DATA_START_ROW + row_idx as u16;
    execute!(out, cursor::MoveTo(0, y), terminal::Clear(ClearType::CurrentLine))?;
    render_data_row(out, item, selected, col)?;
    out.flush()?;
    Ok(())
}

fn render_help_menu(out: &mut impl Write, row: usize, col: usize, rows: &[HelpRow]) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let border = "─".repeat(COL_SHORTCUT + COL_COMMAND + COL_DESCRIPTION + 10);

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$} │\r\n",
            t!("tui.help.title"),
            width = COL_SHORTCUT + COL_COMMAND + COL_DESCRIPTION + 6
        )),
        Print(format!(
            "│ {:<width$} │\r\n",
            " ",
            width = COL_SHORTCUT + COL_COMMAND + COL_DESCRIPTION + 6
        )),
        ResetColor,
    )?;

    render_header(out)?;

    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("├{}┤\r\n", border)),
        ResetColor,
    )?;

    for (idx, item) in rows.iter().enumerate() {
        render_data_row(out, item, idx == row, col)?;
    }

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!(
            "│ {:<width$} │\r\n",
            " ",
            width = COL_SHORTCUT + COL_COMMAND + COL_DESCRIPTION + 6
        )),
        Print(format!(
            "│ {:<width$} │\r\n",
            t!("tui.help.navigation"),
            width = COL_SHORTCUT + COL_COMMAND + COL_DESCRIPTION + 6
        )),
        Print(format!("└{}┘\r\n", border)),
        ResetColor,
    )?;

    out.flush()?;
    Ok(())
}

fn render_header(out: &mut impl Write) -> io::Result<()> {
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print("│ "),
        ResetColor,
    )?;
    render_cell(out, &t!("tui.help.column_shortcut"), COL_SHORTCUT, Color::Yellow, false)?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        ResetColor,
    )?;
    render_cell(out, &t!("tui.help.column_command"), COL_COMMAND, Color::Magenta, false)?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        ResetColor,
    )?;
    render_cell(out, &t!("tui.help.column_description"), COL_DESCRIPTION, Color::Blue, false)?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │\r\n"),
        ResetColor,
    )?;
    Ok(())
}

fn render_data_row(
    out: &mut impl Write,
    row: &HelpRow,
    selected: bool,
    col: usize,
) -> io::Result<()> {
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print("│ "),
        ResetColor,
    )?;
    render_cell(
        out,
        row.shortcut,
        COL_SHORTCUT,
        Color::Yellow,
        selected && col == 0,
    )?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        ResetColor,
    )?;
    render_cell(
        out,
        row.command,
        COL_COMMAND,
        if row.command.ends_with('*') {
            Color::Red
        } else {
            Color::Magenta
        },
        selected && col == 1,
    )?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        ResetColor,
    )?;
    render_cell(
        out,
        &row.description,
        COL_DESCRIPTION,
        Color::Blue,
        selected && col == 2,
    )?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │\r\n"),
        ResetColor,
    )?;
    Ok(())
}

fn render_cell(
    out: &mut impl Write,
    value: &str,
    width: usize,
    color: Color,
    selected: bool,
) -> io::Result<()> {
    let fitted = fit(value, width);
    if selected {
        execute!(
            out,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Print(format!("{:<width$}", fitted, width = width)),
            ResetColor,
        )?;
    } else {
        execute!(
            out,
            SetForegroundColor(color),
            Print(format!("{:<width$}", fitted, width = width)),
            ResetColor,
        )?;
    }
    Ok(())
}

fn fit(value: &str, width: usize) -> String {
    if value.len() <= width {
        return value.to_owned();
    }
    if width <= 3 {
        return value[..width].to_owned();
    }
    format!("{}...", &value[..width - 3])
}
