use crate::shell::config::{ReporterLevel, VisualConfig};
use crate::t;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write, stdout};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigMenuSelection {
    Closed,
    UpdatedVisual(VisualConfig),
    UpdatedReporterLevel(ReporterLevel),
    TomlEditor,
}

struct ConfigRow {
    feature: String,
    action_name: &'static str,
    description: String,
}

const COL_FEATURE: usize = 30;
const COL_ACTION: usize = 20;
const COL_DESCRIPTION: usize = 42;

/// Fila Y (0-based) donde empiezan las filas de datos del menú principal.
/// Estructura: borde(0) título(1) instrucciones(2) cabecera(3) separador(4) datos(5…)
const CONFIG_DATA_START_ROW: u16 = 5;
const FONT_OPTIONS: &[&str] = &[
    "Cascadia Mono",
    "Consolas",
    "JetBrains Mono",
    "Fira Code",
    "Monospace",
];

fn config_rows() -> Vec<ConfigRow> {
    vec![
        ConfigRow {
            feature: t!("tui.config.color_personalization"),
            action_name: "ui.colors",
            description: t!("tui.config.color_desc"),
        },
        ConfigRow {
            feature: t!("tui.config.font_selector"),
            action_name: "ui.fonts",
            description: t!("tui.config.font_desc"),
        },
        ConfigRow {
            feature: t!("tui.config.toml_editor"),
            action_name: "commands.toml",
            description: t!("tui.config.toml_desc"),
        },
        ConfigRow {
            feature: t!("tui.config.reporter_level"),
            action_name: "behavior.reporter_level",
            description: t!("tui.config.reporter_desc"),
        },
    ]
}

const REPORTER_LEVELS: &[ReporterLevel] = &[
    ReporterLevel::Info,
    ReporterLevel::Warning,
    ReporterLevel::Error,
];

struct ColorPreset {
    label: &'static str,
    code: u8,
}

const COLOR_PRESETS: &[ColorPreset] = &[
    ColorPreset {
        label: "Black",
        code: 0,
    },
    ColorPreset {
        label: "Dark Gray",
        code: 240,
    },
    ColorPreset {
        label: "Light Gray",
        code: 253,
    },
    ColorPreset {
        label: "Purple",
        code: 141,
    },
    ColorPreset {
        label: "Pink",
        code: 213,
    },
    ColorPreset {
        label: "Blue",
        code: 39,
    },
    ColorPreset {
        label: "Green",
        code: 46,
    },
    ColorPreset {
        label: "Yellow",
        code: 220,
    },
];

const COLOR_FIELDS: &[&str] = &[
    "Terminal foreground",
    "Terminal background",
    "Prompt path",
    "Prompt subsystem",
    "Prompt name",
    "Prompt dim",
];

pub fn show_config_menu(current_visual: &VisualConfig) -> io::Result<ConfigMenuSelection> {
    show_config_menu_with_behavior(current_visual, ReporterLevel::Error)
}

pub fn show_config_menu_with_behavior(
    current_visual: &VisualConfig,
    current_reporter_level: ReporterLevel,
) -> io::Result<ConfigMenuSelection> {
    let mut out = stdout();
    terminal::enable_raw_mode()?;
    execute!(
        out,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Hide,
    )?;

    let result = run_config_menu(&mut out, current_visual, current_reporter_level);

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

fn run_config_menu(
    out: &mut impl Write,
    current_visual: &VisualConfig,
    current_reporter_level: ReporterLevel,
) -> io::Result<ConfigMenuSelection> {
    let mut row = 0usize;
    let mut col = 0usize;
    let mut visual = current_visual.clone();
    let rows = config_rows();

    // Render inicial completo
    render_config_menu(out, row, col, &rows)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            let prev_row = row;
            let prev_col = col;

            match key.code {
                KeyCode::Up => {
                    row = row.saturating_sub(1);
                }
                KeyCode::Down => {
                    if row + 1 < rows.len() {
                        row += 1;
                    }
                }
                KeyCode::Left => {
                    col = col.saturating_sub(1);
                }
                KeyCode::Right => {
                    if col < 2 {
                        col += 1;
                    }
                }
                KeyCode::Enter => match row {
                    0 => {
                        if let Some(updated) = show_color_editor(out, &visual)? {
                            visual = updated;
                            return Ok(ConfigMenuSelection::UpdatedVisual(visual));
                        }
                        // Submenú cerrado — render completo para restaurar la pantalla
                        render_config_menu(out, row, col, &rows)?;
                    }
                    1 => {
                        if let Some(updated) = show_font_editor(out, &visual)? {
                            visual = updated;
                            return Ok(ConfigMenuSelection::UpdatedVisual(visual));
                        }
                        render_config_menu(out, row, col, &rows)?;
                    }
                    2 => return Ok(ConfigMenuSelection::TomlEditor),
                    3 => {
                        if let Some(updated) = show_reporter_level_editor(out, current_reporter_level)? {
                            return Ok(ConfigMenuSelection::UpdatedReporterLevel(updated));
                        }
                        render_config_menu(out, row, col, &rows)?;
                    }
                    _ => {}
                },
                KeyCode::Esc | KeyCode::Char('q') => {
                    return Ok(ConfigMenuSelection::Closed);
                }
                _ => {}
            }

            // Render diferencial: solo actualiza las filas que cambiaron
            if row != prev_row {
                update_config_row(out, prev_row, &rows[prev_row], false, col)?;
                update_config_row(out, row, &rows[row], true, col)?;
            } else if col != prev_col {
                update_config_row(out, row, &rows[row], true, col)?;
            }
        }
    }
}

fn show_reporter_level_editor(
    out: &mut impl Write,
    current_level: ReporterLevel,
) -> io::Result<Option<ReporterLevel>> {
    let mut selected = REPORTER_LEVELS
        .iter()
        .position(|level| *level == current_level)
        .unwrap_or(2);

    render_reporter_level_editor(out, selected)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            match key.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                        render_reporter_level_editor(out, selected)?;
                    }
                }
                KeyCode::Down => {
                    if selected + 1 < REPORTER_LEVELS.len() {
                        selected += 1;
                        render_reporter_level_editor(out, selected)?;
                    }
                }
                KeyCode::Enter => return Ok(Some(REPORTER_LEVELS[selected])),
                KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                _ => {}
            }
        }
    }
}

fn show_font_editor(
    out: &mut impl Write,
    current_visual: &VisualConfig,
) -> io::Result<Option<VisualConfig>> {
    let mut visual = current_visual.clone();
    let mut selected = FONT_OPTIONS
        .iter()
        .position(|font| *font == visual.font_family)
        .unwrap_or(0);

    render_font_editor(out, selected)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            match key.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                        render_font_editor(out, selected)?;
                    }
                }
                KeyCode::Down => {
                    if selected + 1 < FONT_OPTIONS.len() {
                        selected += 1;
                        render_font_editor(out, selected)?;
                    }
                }
                KeyCode::Enter => {
                    visual.font_family = FONT_OPTIONS[selected].to_owned();
                    return Ok(Some(visual));
                }
                KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                _ => {}
            }
        }
    }
}

fn show_color_editor(
    out: &mut impl Write,
    current_visual: &VisualConfig,
) -> io::Result<Option<VisualConfig>> {
    let mut visual = current_visual.clone();
    let mut row = 0usize;

    render_color_editor(out, &visual, row)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            match key.code {
                KeyCode::Up => {
                    if row > 0 {
                        row -= 1;
                        render_color_editor(out, &visual, row)?;
                    }
                }
                KeyCode::Down => {
                    if row + 1 < COLOR_FIELDS.len() {
                        row += 1;
                        render_color_editor(out, &visual, row)?;
                    }
                }
                KeyCode::Left => {
                    let current_code = color_field_value(&visual, row);
                    let current_idx = preset_index(current_code);
                    let next_idx = current_idx.saturating_sub(1);
                    set_color_field(&mut visual, row, COLOR_PRESETS[next_idx].code);
                    render_color_editor(out, &visual, row)?;
                }
                KeyCode::Right => {
                    let current_code = color_field_value(&visual, row);
                    let current_idx = preset_index(current_code);
                    let next_idx = if current_idx + 1 < COLOR_PRESETS.len() {
                        current_idx + 1
                    } else {
                        current_idx
                    };
                    set_color_field(&mut visual, row, COLOR_PRESETS[next_idx].code);
                    render_color_editor(out, &visual, row)?;
                }
                KeyCode::Enter => return Ok(Some(visual)),
                KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                _ => {}
            }
        }
    }
}

fn render_font_editor(out: &mut impl Write, selected: usize) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let width = 72usize;
    let border = "─".repeat(width - 2);

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$}│\r\n",
            t!("tui.config.font_editor_title"),
            width = width - 3
        )),
        Print(format!(
            "│ {:<width$}│\r\n",
            t!("tui.config.font_editor_help"),
            width = width - 3
        )),
        Print(format!("├{}┤\r\n", border)),
        ResetColor,
    )?;

    for (idx, font) in FONT_OPTIONS.iter().enumerate() {
        if idx == selected {
            execute!(
                out,
                SetBackgroundColor(Color::DarkBlue),
                SetForegroundColor(Color::White),
                Print(format!("│  ❯ {:<width$}│\r\n", font, width = width - 6)),
                ResetColor,
            )?;
        } else {
            execute!(
                out,
                SetForegroundColor(Color::Magenta),
                Print(format!("│    {:<width$}│\r\n", font, width = width - 6)),
                ResetColor,
            )?;
        }
    }

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("└{}┘\r\n", border)),
        ResetColor,
    )?;
    out.flush()?;
    Ok(())
}

fn render_reporter_level_editor(out: &mut impl Write, selected: usize) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let width = 72usize;
    let border = "─".repeat(width - 2);

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$}│\r\n",
            t!("tui.config.reporter_editor_title"),
            width = width - 3
        )),
        Print(format!(
            "│ {:<width$}│\r\n",
            t!("tui.config.reporter_editor_help"),
            width = width - 3
        )),
        Print(format!("├{}┤\r\n", border)),
        ResetColor,
    )?;

    for (idx, level) in REPORTER_LEVELS.iter().enumerate() {
        let label = match level {
            ReporterLevel::Info => t!("tui.config.reporter_level_info"),
            ReporterLevel::Warning => t!("tui.config.reporter_level_warning"),
            ReporterLevel::Error => t!("tui.config.reporter_level_error"),
        };

        if idx == selected {
            execute!(
                out,
                SetBackgroundColor(Color::DarkBlue),
                SetForegroundColor(Color::White),
                Print(format!("│  ❯ {:<width$}│\r\n", label, width = width - 6)),
                ResetColor,
            )?;
        } else {
            execute!(
                out,
                SetForegroundColor(Color::Magenta),
                Print(format!("│    {:<width$}│\r\n", label, width = width - 6)),
                ResetColor,
            )?;
        }
    }

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("└{}┘\r\n", border)),
        ResetColor,
    )?;
    out.flush()?;
    Ok(())
}

fn render_color_editor(
    out: &mut impl Write,
    visual: &VisualConfig,
    selected: usize,
) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let width = 92usize;
    let border = "─".repeat(width - 2);

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$}│\r\n",
            t!("tui.config.color_editor_title"),
            width = width - 3
        )),
        Print(format!(
            "│ {:<width$}│\r\n",
            t!("tui.config.color_editor_help"),
            width = width - 3
        )),
        Print(format!("├{}┤\r\n", border)),
        ResetColor,
    )?;

    for (idx, label) in COLOR_FIELDS.iter().enumerate() {
        let code = color_field_value(visual, idx);
        let preset = &COLOR_PRESETS[preset_index(code)];
        let line = format!("{:<24} {:<16} ansi:{:<3}", label, preset.label, preset.code);

        if idx == selected {
            execute!(
                out,
                SetBackgroundColor(Color::DarkBlue),
                SetForegroundColor(Color::White),
                Print(format!("│  ❯ {:<width$}│\r\n", line, width = width - 6)),
                ResetColor,
            )?;
        } else {
            execute!(
                out,
                SetForegroundColor(Color::Yellow),
                Print(format!("│    {:<width$}│\r\n", line, width = width - 6)),
                ResetColor,
            )?;
        }
    }

    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("│ {:<width$}│\r\n", " ", width = width - 3)),
        Print(format!(
            "│ {:<width$}│\r\n",
            t!("tui.config.preview"),
            width = width - 3
        )),
        ResetColor,
    )?;

    execute!(
        out,
        SetForegroundColor(Color::AnsiValue(visual.prompt_path_ansi256)),
        Print("│ "),
        Print("path"),
        ResetColor,
        SetForegroundColor(Color::DarkGrey),
        Print(" · "),
        ResetColor,
        SetForegroundColor(Color::AnsiValue(visual.prompt_subsystem_ansi256)),
        Print("subsystem"),
        ResetColor,
        SetForegroundColor(Color::DarkGrey),
        Print(" · "),
        ResetColor,
        SetForegroundColor(Color::AnsiValue(visual.prompt_name_ansi256)),
        Print("name"),
        ResetColor,
        SetForegroundColor(Color::DarkGrey),
        Print(" · "),
        ResetColor,
        SetForegroundColor(Color::AnsiValue(visual.prompt_dim_ansi256)),
        Print("dim"),
        ResetColor,
        Print(format!("\r\n└{}┘\r\n", border)),
    )?;

    out.flush()?;
    Ok(())
}

/// Actualiza una única fila de datos en su posición Y exacta.
/// Evita el Clear(All) completo — elimina el flicker en navegación.
fn update_config_row(
    out: &mut impl Write,
    row_idx: usize,
    item: &ConfigRow,
    selected: bool,
    col: usize,
) -> io::Result<()> {
    let y = CONFIG_DATA_START_ROW + row_idx as u16;
    execute!(
        out,
        cursor::MoveTo(0, y),
        terminal::Clear(ClearType::CurrentLine)
    )?;
    render_data_row(out, item, selected, col)?;
    out.flush()?;
    Ok(())
}

fn render_config_menu(
    out: &mut impl Write,
    row: usize,
    col: usize,
    rows: &[ConfigRow],
) -> io::Result<()> {
    execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All),)?;

    let border = "─".repeat(COL_FEATURE + COL_ACTION + COL_DESCRIPTION + 10);
    let content_width = COL_FEATURE + COL_ACTION + COL_DESCRIPTION + 6;

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\r\n", border)),
        Print(format!(
            "│ {:<width$} │\r\n",
            t!("tui.config.menu_title"),
            width = content_width
        )),
        Print(format!(
            "│ {:<width$} │\r\n",
            t!("tui.config.menu_help"),
            width = content_width
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
        Print(format!("│ {:<width$} │\r\n", " ", width = content_width)),
        SetBackgroundColor(Color::DarkRed),
        SetForegroundColor(Color::Yellow),
        Print(format!(
            "│ {:<width$} │\r\n",
            t!("tui.config.toml_warning"),
            width = content_width
        )),
        ResetColor,
        SetForegroundColor(Color::Cyan),
        Print(format!(
            "│ {:<width$} │\r\n",
            t!("tui.config.menu_navigation"),
            width = content_width
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
    render_cell(
        out,
        &t!("tui.config.column_feature"),
        COL_FEATURE,
        Color::Yellow,
        false,
    )?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        ResetColor,
    )?;
    render_cell(
        out,
        &t!("tui.config.column_action"),
        COL_ACTION,
        Color::Magenta,
        false,
    )?;
    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        ResetColor,
    )?;
    render_cell(
        out,
        &t!("tui.config.column_description"),
        COL_DESCRIPTION,
        Color::Blue,
        false,
    )?;
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
    row: &ConfigRow,
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
        &row.feature,
        COL_FEATURE,
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
        row.action_name,
        COL_ACTION,
        Color::Magenta,
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

fn color_field_value(visual: &VisualConfig, row: usize) -> u8 {
    match row {
        0 => visual.terminal_foreground_ansi256,
        1 => visual.terminal_background_ansi256,
        2 => visual.prompt_path_ansi256,
        3 => visual.prompt_subsystem_ansi256,
        4 => visual.prompt_name_ansi256,
        _ => visual.prompt_dim_ansi256,
    }
}

fn set_color_field(visual: &mut VisualConfig, row: usize, value: u8) {
    match row {
        0 => visual.terminal_foreground_ansi256 = value,
        1 => visual.terminal_background_ansi256 = value,
        2 => visual.prompt_path_ansi256 = value,
        3 => visual.prompt_subsystem_ansi256 = value,
        4 => visual.prompt_name_ansi256 = value,
        _ => visual.prompt_dim_ansi256 = value,
    }
}

fn preset_index(code: u8) -> usize {
    COLOR_PRESETS
        .iter()
        .position(|preset| preset.code == code)
        .unwrap_or(0)
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
