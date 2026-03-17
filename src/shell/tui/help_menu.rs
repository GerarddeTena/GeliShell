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
    description: &'static str,
    action: HelpMenuAction,
}

const COL_SHORTCUT: usize = 12;
const COL_COMMAND: usize = 14;
const COL_DESCRIPTION: usize = 66;

const HELP_ROWS: &[HelpRow] = &[
    HelpRow {
        shortcut: "^C",
        command: ":stop*",
        description: "interrumpe el proceso actual (alias estilo vim/cmd-mode)",
        action: HelpMenuAction::Stop,
    },
    HelpRow {
        shortcut: "^D",
        command: "exit",
        description: "cierra GeliShell y termina el proceso de la terminal",
        action: HelpMenuAction::Exit,
    },
    HelpRow {
        shortcut: "^L",
        command: "clear",
        description: "limpia pantalla + scrollback (buffer real, tipo cls)",
        action: HelpMenuAction::Clear,
    },
    HelpRow {
        shortcut: "^S",
        command: ":search*",
        description: "intercepta búsqueda rápida de comandos y acciones",
        action: HelpMenuAction::Search,
    },
];

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

    render_help_menu(out, row, col)?;

    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                match key.code {
                    KeyCode::Up => {
                        if row > 0 {
                            row -= 1;
                            render_help_menu(out, row, col)?;
                        }
                    }
                    KeyCode::Down => {
                        if row + 1 < HELP_ROWS.len() {
                            row += 1;
                            render_help_menu(out, row, col)?;
                        }
                    }
                    KeyCode::Left => {
                        if col > 0 {
                            col -= 1;
                            render_help_menu(out, row, col)?;
                        }
                    }
                    KeyCode::Right => {
                        if col < 2 {
                            col += 1;
                            render_help_menu(out, row, col)?;
                        }
                    }
                    KeyCode::Enter => {
                        return Ok(HELP_ROWS[row].action);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        return Ok(HelpMenuAction::None);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn render_help_menu(out: &mut impl Write, row: usize, col: usize) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let border = "─".repeat(COL_SHORTCUT + COL_COMMAND + COL_DESCRIPTION + 10);

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$} │\r\n",
            "Geli HelpMe — atajos y comandos interactivos",
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

    for (idx, item) in HELP_ROWS.iter().enumerate() {
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
            "↑↓ filas · ←→ columnas · Enter ejecutar · Esc cerrar",
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
    render_cell(out, "Atajo", COL_SHORTCUT, Color::Yellow, false)?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        ResetColor,
    )?;
    render_cell(out, "Comando", COL_COMMAND, Color::Magenta, false)?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        ResetColor,
    )?;
    render_cell(out, "Descripcion", COL_DESCRIPTION, Color::Blue, false)?;
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
        row.description,
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
