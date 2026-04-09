mod error;

use crate::shell::{
    commands::ecosystems::{EcosystemCatalog, EcosystemCommand},
    reporter::Reporter,
    translator::Subsystem,
    tui::show_me::{resolve_placeholders_for_tui, subsystem_matches_for_tui},
};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{self, ClearType},
};
pub(crate) use error::EcosystemTuiError;
use std::io::{self, Write, stdout};

const HELP_ROW_PADDING: u16 = 2;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Panel {
    Operations,
    Commands,
}

#[derive(Clone)]
struct VisibleOperation {
    operation: String,
    level: String,
    commands: Vec<EcosystemCommand>,
}

pub struct EcosystemTui {
    catalog: EcosystemCatalog,
    subsystem: Subsystem,
    active_panel: Panel,
    op_selected: usize,
    cmd_selected: usize,
    filter: String,
    filter_mode: bool,
    subsys_filter: bool,
    theme_color: Color,
    icon: &'static str,
}

impl EcosystemTui {
    pub fn new(catalog: EcosystemCatalog, subsystem: Subsystem) -> Self {
        let (theme_color, icon) = get_theme(&catalog.meta.name);
        Self {
            catalog,
            subsystem,
            active_panel: Panel::Operations,
            op_selected: 0,
            cmd_selected: 0,
            filter: String::new(),
            filter_mode: false,
            subsys_filter: true,
            theme_color,
            icon,
        }
    }

    pub async fn run(
        &mut self,
        _reporter: &dyn Reporter,
    ) -> Result<Option<String>, EcosystemTuiError> {
        let mut out = stdout();
        terminal::enable_raw_mode().map_err(terminal_error)?;

        execute!(
            out,
            terminal::EnterAlternateScreen,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide
        )
        .map_err(terminal_error)?;

        let result = self.run_loop(&mut out);

        let cleanup_screen = execute!(
            out,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Show,
            terminal::LeaveAlternateScreen
        )
        .map_err(terminal_error);
        let cleanup_raw = terminal::disable_raw_mode().map_err(terminal_error);

        match result {
            Ok(value) => {
                cleanup_screen?;
                cleanup_raw?;
                Ok(value)
            }
            Err(error) => {
                let _ = cleanup_screen;
                let _ = cleanup_raw;
                Err(error)
            }
        }
    }

    fn run_loop(&mut self, out: &mut impl Write) -> Result<Option<String>, EcosystemTuiError> {
        self.render(out)?;

        loop {
            let event = event::read().map_err(terminal_error)?;
            let Event::Key(key) = event else {
                continue;
            };

            if key.kind != KeyEventKind::Press {
                continue;
            }

            if self.filter_mode {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => self.filter_mode = false,
                    KeyCode::Backspace => {
                        self.filter.pop();
                        self.op_selected = 0;
                        self.cmd_selected = 0;
                    }
                    KeyCode::Char(ch) => {
                        self.filter.push(ch);
                        self.op_selected = 0;
                        self.cmd_selected = 0;
                    }
                    _ => {}
                }
                self.render(out)?;
                continue;
            }

            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(None),
                KeyCode::Tab => {
                    self.active_panel = match self.active_panel {
                        Panel::Operations => Panel::Commands,
                        Panel::Commands => Panel::Operations,
                    };
                    self.cmd_selected = 0;
                }
                KeyCode::Char('/') => {
                    self.filter_mode = true;
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.subsys_filter = !self.subsys_filter;
                    self.cmd_selected = 0;
                }
                KeyCode::Up => self.navigate_up(),
                KeyCode::Down => self.navigate_down(),
                KeyCode::Enter if self.active_panel == Panel::Commands => {
                    if let Some(command) = self.current_selected_command() {
                        let resolved = resolve_placeholders_for_tui(&command, out)
                            .map_err(|e| EcosystemTuiError::Terminal(e.to_string()))?;
                        if self.confirm_execution(out, &resolved)? {
                            return Ok(Some(resolved));
                        }
                    }
                }
                _ => {}
            }

            self.render(out)?;
        }
    }

    fn navigate_up(&mut self) {
        match self.active_panel {
            Panel::Operations => {
                if self.op_selected > 0 {
                    self.op_selected -= 1;
                    self.cmd_selected = 0;
                }
            }
            Panel::Commands => {
                if self.cmd_selected > 0 {
                    self.cmd_selected -= 1;
                }
            }
        }
    }

    fn navigate_down(&mut self) {
        match self.active_panel {
            Panel::Operations => {
                let total = self.visible_operations().len();
                if self.op_selected + 1 < total {
                    self.op_selected += 1;
                    self.cmd_selected = 0;
                }
            }
            Panel::Commands => {
                let total = self.current_commands().len();
                if self.cmd_selected + 1 < total {
                    self.cmd_selected += 1;
                }
            }
        }
    }

    fn visible_operations(&self) -> Vec<VisibleOperation> {
        let filter = self.filter.trim().to_ascii_lowercase();
        self.catalog
            .ops
            .iter()
            .filter(|op| {
                if filter.is_empty() {
                    true
                } else {
                    op.operation.to_ascii_lowercase().contains(&filter)
                }
            })
            .map(|op| VisibleOperation {
                operation: op.operation.clone(),
                level: op.level.clone(),
                commands: op.commands.clone(),
            })
            .collect()
    }

    fn current_commands(&self) -> Vec<EcosystemCommand> {
        let ops = self.visible_operations();
        let Some(op) = ops.get(self.op_selected) else {
            return Vec::new();
        };

        if !self.subsys_filter {
            return op.commands.clone();
        }

        let active = self.subsystem.as_str();
        let filtered: Vec<EcosystemCommand> = op
            .commands
            .iter()
            .filter(|cmd| subsystem_matches_for_tui(&cmd.subsystem, active))
            .cloned()
            .collect();

        if filtered.is_empty() {
            op.commands.clone()
        } else {
            filtered
        }
    }

    fn current_selected_command(&self) -> Option<String> {
        let commands = self.current_commands();
        commands
            .get(self.cmd_selected)
            .map(|entry| entry.command.clone())
    }

    fn render(&mut self, out: &mut impl Write) -> Result<(), EcosystemTuiError> {
        let (width, height) = terminal::size().map_err(terminal_error)?;
        let left_w = width / 3;
        let mid_w = width / 3;
        let right_w = width.saturating_sub(left_w + mid_w + 2);

        let operations = self.visible_operations();
        if self.op_selected >= operations.len() && !operations.is_empty() {
            self.op_selected = operations.len() - 1;
            self.cmd_selected = 0;
        }

        let commands = self.current_commands();
        if self.cmd_selected >= commands.len() && !commands.is_empty() {
            self.cmd_selected = commands.len() - 1;
        }

        execute!(
            out,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All),
            SetForegroundColor(self.theme_color),
            SetAttribute(Attribute::Bold),
            Print(format!(
                " {} GeliShell Ecosystem — {} ",
                self.icon,
                self.catalog.meta.name.to_uppercase()
            )),
            SetAttribute(Attribute::Reset),
            Print(format!(" :: {} commands loaded", self.catalog.ops.len())),
            Print("\r\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("─".repeat(width as usize)),
            ResetColor,
            Print("\r\n")
        )
        .map_err(terminal_error)?;

        self.write_cell(
            out,
            0,
            2,
            left_w,
            "Operations",
            self.active_panel == Panel::Operations,
        )?;
        self.write_cell(
            out,
            left_w + 1,
            2,
            mid_w,
            "Commands",
            self.active_panel == Panel::Commands,
        )?;
        self.write_cell(out, left_w + mid_w + 2, 2, right_w, "Detail", false)?;

        let max_rows = height.saturating_sub(7 + HELP_ROW_PADDING) as usize;

        for row in 0..max_rows {
            let y = 4 + row as u16;

            let op_text = operations
                .get(row)
                .map(|op| op.operation.as_str())
                .unwrap_or("");
            let op_line = if row == self.op_selected {
                format!("> {}", truncate(op_text, left_w as usize - 2))
            } else {
                format!("  {}", truncate(op_text, left_w as usize - 2))
            };
            self.write_row(
                out,
                0,
                y,
                left_w,
                &op_line,
                self.active_panel == Panel::Operations && row == self.op_selected,
            )?;

            let cmd_text = commands
                .get(row)
                .map(|cmd| cmd.command.as_str())
                .unwrap_or("");
            let cmd_line = if row == self.cmd_selected {
                format!("> {}", truncate(cmd_text, mid_w as usize - 2))
            } else {
                format!("  {}", truncate(cmd_text, mid_w as usize - 2))
            };
            self.write_row(
                out,
                left_w + 1,
                y,
                mid_w,
                &cmd_line,
                self.active_panel == Panel::Commands && row == self.cmd_selected,
            )?;

            let detail = self.detail_line(row, &operations, &commands);
            self.write_row(out, left_w + mid_w + 2, y, right_w, &detail, false)?;
        }

        // Draw verticals separators
        for row in 2..(height - 3) {
            execute!(out, SetForegroundColor(Color::DarkGrey)).map_err(terminal_error)?;
            execute!(out, cursor::MoveTo(left_w, row), Print("│")).map_err(terminal_error)?;
            execute!(out, cursor::MoveTo(left_w + mid_w + 1, row), Print("│"))
                .map_err(terminal_error)?;
        }

        // Footer
        let filter_text = if self.filter.is_empty() {
            "".to_string()
        } else {
            format!(" Filter: [{}]", self.filter)
        };
        let subsys_label = if self.subsys_filter {
            format!("Subsystem: {} (s)", self.subsystem.as_str())
        } else {
            "ALL (s)".to_owned()
        };

        execute!(
            out,
            cursor::MoveTo(0, height.saturating_sub(2)),
            SetForegroundColor(Color::DarkGrey),
            Print("─".repeat(width as usize)),
            ResetColor,
            cursor::MoveTo(0, height.saturating_sub(1)),
            Print(" "),
            SetForegroundColor(self.theme_color),
            Print("TAB"),
            ResetColor,
            Print(" Panel  "),
            SetForegroundColor(self.theme_color),
            Print("RET"),
            ResetColor,
            Print(" Exec  "),
            SetForegroundColor(self.theme_color),
            Print("s"),
            ResetColor,
            Print(format!(" {} ", subsys_label)),
            SetForegroundColor(self.theme_color),
            Print("/"),
            ResetColor,
            Print(" Filter "),
            SetForegroundColor(self.theme_color),
            Print("Q"),
            ResetColor,
            Print(" Quit  "),
            Print(filter_text)
        )
        .map_err(terminal_error)?;

        out.flush()?;
        Ok(())
    }

    fn detail_line(
        &self,
        row: usize,
        operations: &[VisibleOperation],
        commands: &[EcosystemCommand],
    ) -> String {
        let Some(op) = operations.get(self.op_selected) else {
            return String::new();
        };

        match row {
            0 => format!("Operation: {}", op.operation),
            1 => format!("Level: {}", op.level),
            2 => format!("Subsystem: {}", self.subsystem.as_str()),
            3 => format!("Description: {}", self.catalog.meta.description),
            4 => format!("Visible commands: {}", commands.len()),
            _ => String::new(),
        }
    }

    fn write_cell(
        &self,
        out: &mut impl Write,
        x: u16,
        y: u16,
        width: u16,
        title: &str,
        active: bool,
    ) -> Result<(), EcosystemTuiError> {
        let rendered = format!("{title:<width$}", width = width as usize);
        execute!(out, cursor::MoveTo(x, y)).map_err(terminal_error)?;
        if active {
            execute!(
                out,
                SetAttribute(Attribute::Reverse),
                Print(rendered),
                SetAttribute(Attribute::Reset)
            )
            .map_err(terminal_error)?;
        } else {
            execute!(
                out,
                SetForegroundColor(self.theme_color),
                Print(rendered),
                ResetColor
            )
            .map_err(terminal_error)?;
        }
        Ok(())
    }

    fn write_row(
        &self,
        out: &mut impl Write,
        x: u16,
        y: u16,
        width: u16,
        text: &str,
        highlight: bool,
    ) -> Result<(), EcosystemTuiError> {
        let rendered = format!(
            "{:<width$}",
            truncate(text, width as usize),
            width = width as usize
        );
        execute!(out, cursor::MoveTo(x, y)).map_err(terminal_error)?;
        if highlight {
            execute!(
                out,
                SetBackgroundColor(self.theme_color),
                SetForegroundColor(Color::Black),
                Print(rendered),
                ResetColor
            )
            .map_err(terminal_error)?;
        } else {
            execute!(out, Print(rendered)).map_err(terminal_error)?;
        }
        Ok(())
    }

    fn confirm_execution(
        &mut self,
        out: &mut impl Write,
        command: &str,
    ) -> Result<bool, EcosystemTuiError> {
        terminal::disable_raw_mode().map_err(terminal_error)?;
        execute!(out, cursor::Show, terminal::LeaveAlternateScreen).map_err(terminal_error)?;

        let prompt_result = (|| -> Result<bool, EcosystemTuiError> {
            writeln!(out)?;
            writeln!(out, "{command}")?;
            writeln!(out)?;
            write!(out, "Execute? [y/N]: ")?;
            out.flush()?;

            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            let normalized = answer.trim().to_ascii_lowercase();
            Ok(normalized == "y" || normalized == "yes")
        })();

        execute!(
            out,
            terminal::EnterAlternateScreen,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide
        )
        .map_err(terminal_error)?;
        terminal::enable_raw_mode().map_err(terminal_error)?;

        prompt_result
    }
}

fn get_theme(name: &str) -> (Color, &'static str) {
    match name.to_ascii_lowercase().as_str() {
        "npm" => (
            Color::Rgb {
                r: 203,
                g: 56,
                b: 55,
            },
            "📦",
        ),
        "git" => (
            Color::Rgb {
                r: 241,
                g: 78,
                b: 50,
            },
            "🐈‍",
        ),
        "cargo" => (
            Color::Rgb {
                r: 222,
                g: 165,
                b: 132,
            },
            "🦀",
        ),
        "docker" => (
            Color::Rgb {
                r: 36,
                g: 150,
                b: 237,
            },
            "🐳",
        ),
        "dotnet" => (
            Color::Rgb {
                r: 81,
                g: 43,
                b: 212,
            },
            "🟣",
        ),
        "node" => (
            Color::Rgb {
                r: 104,
                g: 160,
                b: 99,
            },
            "✅",
        ),
        "pnpm" => (
            Color::Rgb {
                r: 246,
                g: 146,
                b: 32,
            },
            "🟠",
        ),
        "python" => (
            Color::Rgb {
                r: 75,
                g: 139,
                b: 190,
            },
            "🐍",
        ),
        "typescript" => (
            Color::Rgb {
                r: 49,
                g: 120,
                b: 198,
            },
            "🔷",
        ),
        _ => (
            Color::Rgb {
                r: 0,
                g: 200,
                b: 220,
            },
            "⚡",
        ),
    }
}

fn terminal_error(error: io::Error) -> EcosystemTuiError {
    EcosystemTuiError::Terminal(error.to_string())
}

fn truncate(value: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }

    if value.len() <= max {
        return value.to_owned();
    }

    if max <= 3 {
        return value[..max].to_owned();
    }

    format!("{}...", &value[..max - 3])
}
