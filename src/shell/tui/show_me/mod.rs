mod catalog;
mod db;
mod error;
mod placeholder;

use self::{
    catalog::{CatalogTree, build_catalog},
    db::DocsDb,
    placeholder::resolve_placeholders,
};
use crate::shell::{
    config::VisualConfig,
    reporter::{Reporter, SilentReporter},
    translator::Subsystem,
};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write, stdout};

pub use self::error::ShowMeError;

const MODAL_HEIGHT: u16 = 16;
const MAX_MODAL_ROWS: usize = 8;

#[derive(Debug, Clone)]
enum ShowMeState {
    CategoryList {
        selected: usize,
    },
    CommandTable {
        category: String,
        selected: usize,
        scroll: usize,
    },
    Exit,
}

#[derive(Debug, Clone)]
struct CommandEntry {
    operation: String,
    subsystem: String,
    command: String,
}

pub(crate) struct ShowMeTui<'a> {
    catalog: &'a CatalogTree,
    visual: &'a VisualConfig,
    subsystem: Subsystem,
    state: ShowMeState,
    modal_start_row: Option<u16>,
}

pub fn run_show_me_tui(
    reporter: &dyn Reporter,
    visual: &VisualConfig,
) -> Result<Option<String>, ShowMeError> {
    let db_path = DocsDb::resolve_path();
    let rows = match DocsDb::load(&db_path) {
        Ok(rows) => rows,
        Err(ShowMeError::DbNotFound { path }) => {
            reporter.warn(&format!(
                "assistant --show-me: docs.db was not found at '{path}'"
            ));
            return Ok(None);
        }
        Err(error) => return Err(error),
    };

    let catalog = build_catalog(&rows);
    if catalog.ops.is_empty() {
        reporter.warn("assistant --show-me: catalog is empty");
        return Ok(None);
    }

    let mut tui = ShowMeTui::new(&catalog, visual);
    tui.run(reporter)
}

impl<'a> ShowMeTui<'a> {
    pub(crate) fn new(catalog: &'a CatalogTree, visual: &'a VisualConfig) -> Self {
        let subsystem = Subsystem::detect(&SilentReporter::new());
        Self {
            catalog,
            visual,
            subsystem,
            state: ShowMeState::CategoryList { selected: 0 },
            modal_start_row: None,
        }
    }

    #[must_use]
    pub(crate) fn run(&mut self, reporter: &dyn Reporter) -> Result<Option<String>, ShowMeError> {
        if self.catalog.ops.is_empty() {
            return Err(ShowMeError::EmptyCatalog);
        }

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

        let result = self.run_loop(&mut out, reporter);

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

    fn run_loop(
        &mut self,
        out: &mut impl Write,
        _reporter: &dyn Reporter,
    ) -> Result<Option<String>, ShowMeError> {
        self.render_current_state(out)?;

        loop {
            let event = event::read().map_err(terminal_error)?;
            let Event::Key(key) = event else {
                continue;
            };

            if key.kind != KeyEventKind::Press {
                continue;
            }

            match self.state.clone() {
                ShowMeState::CategoryList { .. } => self.handle_category_key(key.code, out)?,
                ShowMeState::CommandTable { .. } => {
                    if let Some(command) = self.handle_command_key(key.code, out)? {
                        return Ok(Some(command));
                    }
                }
                ShowMeState::Exit => return Ok(None),
            }

            if matches!(self.state, ShowMeState::Exit) {
                return Ok(None);
            }

            self.render_current_state(out)?;
        }
    }

    fn handle_category_key(
        &mut self,
        key_code: KeyCode,
        _out: &mut impl Write,
    ) -> Result<(), ShowMeError> {
        let total_categories = self.catalog.ops.len();
        if total_categories == 0 {
            self.state = ShowMeState::Exit;
            return Ok(());
        }

        let ShowMeState::CategoryList { selected } = &mut self.state else {
            return Ok(());
        };

        match key_code {
            KeyCode::Up => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            KeyCode::Down => {
                if *selected + 1 < total_categories {
                    *selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some((category, _)) = self.catalog.ops.get_index(*selected) {
                    self.modal_start_row = None;
                    self.state = ShowMeState::CommandTable {
                        category: category.clone(),
                        selected: 0,
                        scroll: 0,
                    };
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.state = ShowMeState::Exit;
            }
            KeyCode::Char(ch) if ch == 'q' || ch == 'Q' => {
                self.state = ShowMeState::Exit;
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_command_key(
        &mut self,
        key_code: KeyCode,
        out: &mut impl Write,
    ) -> Result<Option<String>, ShowMeError> {
        let ShowMeState::CommandTable {
            category,
            selected,
            scroll,
        } = self.state.clone()
        else {
            return Ok(None);
        };

        let (entries, _) = self.filtered_entries(&category);
        let total_entries = entries.len();

        match key_code {
            KeyCode::Up => {
                if let ShowMeState::CommandTable {
                    selected, scroll, ..
                } = &mut self.state
                {
                    if *selected > 0 {
                        *selected -= 1;
                    }
                    if *selected < *scroll {
                        *scroll = *selected;
                    }
                }
            }
            KeyCode::Down => {
                if let ShowMeState::CommandTable {
                    selected, scroll, ..
                } = &mut self.state
                {
                    if *selected + 1 < total_entries {
                        *selected += 1;
                    }
                    if *selected >= *scroll + MAX_MODAL_ROWS {
                        *scroll = *selected + 1 - MAX_MODAL_ROWS;
                    }
                }
            }
            KeyCode::Enter => {
                if total_entries == 0 {
                    return Ok(None);
                }

                let selected_index = selected.min(total_entries.saturating_sub(1));
                let command = entries[selected_index].command.clone();
                let resolved = resolve_placeholders(&command, out)?;
                let should_execute = self.confirm_execution(out, &resolved)?;
                if should_execute {
                    return Ok(Some(resolved));
                }

                self.modal_start_row = None;
            }
            KeyCode::Esc | KeyCode::Backspace => {
                let category_index = self.catalog.ops.get_index_of(&category).unwrap_or(0);
                self.state = ShowMeState::CategoryList {
                    selected: category_index,
                };
                self.modal_start_row = None;
            }
            KeyCode::Char(ch) if ch == 'q' || ch == 'Q' => {
                let category_index = self.catalog.ops.get_index_of(&category).unwrap_or(0);
                self.state = ShowMeState::CategoryList {
                    selected: category_index,
                };
                self.modal_start_row = None;
            }
            _ => {
                if let ShowMeState::CommandTable {
                    category: _,
                    selected: state_selected,
                    scroll: state_scroll,
                } = &mut self.state
                {
                    *state_selected = selected;
                    *state_scroll = scroll;
                }
            }
        }

        Ok(None)
    }

    fn render_current_state(&mut self, out: &mut impl Write) -> Result<(), ShowMeError> {
        match self.state {
            ShowMeState::CategoryList { .. } => self.render_category_list(out),
            ShowMeState::CommandTable { .. } => self.render_command_table(out),
            ShowMeState::Exit => Ok(()),
        }
    }

    fn render_category_list(&self, out: &mut impl Write) -> Result<(), ShowMeError> {
        let selected = match &self.state {
            ShowMeState::CategoryList { selected } => *selected,
            ShowMeState::CommandTable { category, .. } => {
                self.catalog.ops.get_index_of(category).unwrap_or(0)
            }
            ShowMeState::Exit => 0,
        };

        execute!(
            out,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All),
            SetForegroundColor(Color::AnsiValue(self.visual.prompt_name_ansi256)),
            Print(" 󰊠  GeliShell Assistant "),
            SetForegroundColor(Color::AnsiValue(self.visual.prompt_dim_ansi256)),
            Print("--show-me\r\n"),
            ResetColor,
            SetForegroundColor(Color::AnsiValue(self.visual.terminal_foreground_ansi256)),
            Print(" Select a category:\r\n\r\n")
        )
        .map_err(terminal_error)?;

        for (index, (category, _)) in self.catalog.ops.iter().enumerate() {
            if index == selected {
                execute!(
                    out,
                    SetForegroundColor(Color::AnsiValue(self.visual.prompt_subsystem_ansi256)),
                    Print(format!(" 󰄾 {category}\r\n")),
                    ResetColor
                )
                .map_err(terminal_error)?;
            } else {
                execute!(
                    out,
                    SetForegroundColor(Color::AnsiValue(self.visual.prompt_dim_ansi256)),
                    Print(format!("   {category}\r\n")),
                    ResetColor
                )
                .map_err(terminal_error)?;
            }
        }

        execute!(
            out,
            Print("\r\n"),
            SetForegroundColor(Color::AnsiValue(self.visual.prompt_dim_ansi256)),
            Print(" ↑↓ move  ·  Enter open  ·  Esc/Backspace/q exit\r\n"),
            ResetColor
        )
        .map_err(terminal_error)?;

        out.flush()?;
        Ok(())
    }

    fn render_command_table(&mut self, out: &mut impl Write) -> Result<(), ShowMeError> {
        let ShowMeState::CommandTable {
            category,
            mut selected,
            mut scroll,
        } = self.state.clone()
        else {
            return Ok(());
        };

        let (entries, using_subsystem_filter) = self.filtered_entries(&category);
        let total_entries = entries.len();
        if total_entries == 0 {
            selected = 0;
            scroll = 0;
        } else {
            if selected >= total_entries {
                selected = total_entries - 1;
            }
            if selected < scroll {
                scroll = selected;
            }
            if selected >= scroll + MAX_MODAL_ROWS {
                scroll = selected + 1 - MAX_MODAL_ROWS;
            }
        }

        self.state = ShowMeState::CommandTable {
            category: category.clone(),
            selected,
            scroll,
        };

        let start_row = self.ensure_modal_anchor(out)?;
        self.clear_modal_area(out, start_row, MODAL_HEIGHT)?;

        let levels = self.current_levels(&category);
        let filter_note = if using_subsystem_filter {
            ""
        } else {
            "(sin filtro de subsistema)"
        };

        let border_color = Color::AnsiValue(self.visual.prompt_subsystem_ansi256);
        let title_color = Color::AnsiValue(self.visual.prompt_name_ansi256);
        let dim_color = Color::AnsiValue(self.visual.prompt_dim_ansi256);

        execute!(
            out,
            cursor::MoveTo(0, start_row),
            SetForegroundColor(border_color),
            Print("╭─ "),
            SetForegroundColor(title_color),
            Print(format!("Category: {category}")),
            SetForegroundColor(border_color),
            Print("\r\n"),
            ResetColor,
            cursor::MoveTo(0, start_row + 1),
            SetForegroundColor(border_color),
            Print("│ "),
            SetForegroundColor(dim_color),
            Print(format!("Levels: {levels}\r\n")),
            ResetColor,
            cursor::MoveTo(0, start_row + 2),
            SetForegroundColor(border_color),
            Print("│ "),
            SetForegroundColor(dim_color),
            Print(format!(
                "Subsystem: {} {filter_note}\r\n",
                self.subsystem.as_str()
            )),
            ResetColor,
            cursor::MoveTo(0, start_row + 3),
            SetForegroundColor(border_color),
            Print("│\r\n"),
            ResetColor
        )
        .map_err(terminal_error)?;

        let rendered_rows = if entries.is_empty() {
            execute!(
                out,
                cursor::MoveTo(0, start_row + 4),
                SetForegroundColor(border_color),
                Print("│ "),
                SetForegroundColor(dim_color),
                Print("No commands found for this category.\r\n"),
                ResetColor
            )
            .map_err(terminal_error)?;
            1usize
        } else {
            let visible_end = (scroll + MAX_MODAL_ROWS).min(entries.len());
            for (row_offset, absolute_index) in (scroll..visible_end).enumerate() {
                let entry = &entries[absolute_index];
                let is_selected = absolute_index == selected;
                let marker = if is_selected { "󰄾" } else { " " };
                let line = truncate(
                    &format!("{marker} [{}] {}", entry.operation, entry.command),
                    108,
                );
                let row = start_row + 4 + row_offset as u16;

                execute!(out, cursor::MoveTo(0, row)).map_err(terminal_error)?;

                if is_selected {
                    // Highlighted row
                    execute!(
                        out,
                        SetForegroundColor(border_color),
                        Print("│ "),
                        SetAttribute(Attribute::Reverse),
                        SetForegroundColor(Color::AnsiValue(self.visual.prompt_path_ansi256)), // foreground used as bg due to reverse
                        Print(format!("{line:<100}")), // Padding for consistent highlight bar
                        SetAttribute(Attribute::Reset),
                        Print("\r\n")
                    )
                    .map_err(terminal_error)?;
                } else {
                    execute!(
                        out,
                        SetForegroundColor(border_color),
                        Print("│ "),
                        SetForegroundColor(dim_color),
                        Print(format!("{line}\r\n")),
                        ResetColor
                    )
                    .map_err(terminal_error)?;
                }
            }
            visible_end.saturating_sub(scroll)
        };

        for blank_offset in rendered_rows..MAX_MODAL_ROWS {
            let row = start_row + 4 + blank_offset as u16;
            execute!(
                out,
                cursor::MoveTo(0, row),
                SetForegroundColor(border_color),
                Print("│\r\n"),
                ResetColor
            )
            .map_err(terminal_error)?;
        }

        let footer_row = start_row + 4 + MAX_MODAL_ROWS as u16;

        execute!(
            out,
            cursor::MoveTo(0, footer_row),
            SetForegroundColor(border_color),
            Print("│ "),
            SetForegroundColor(dim_color),
            Print("Enter select command  ·  Esc/Backspace/q back\r\n"),
            ResetColor,
            cursor::MoveTo(0, footer_row + 1),
            SetForegroundColor(border_color),
            Print("╰────────────────────────────────────────────────────────────────────────────────────────\r\n"),
            ResetColor
        )
        .map_err(terminal_error)?;

        out.flush()?;
        Ok(())
    }

    fn ensure_modal_anchor(&mut self, _out: &mut impl Write) -> Result<u16, ShowMeError> {
        if let Some(start_row) = self.modal_start_row {
            return Ok(start_row);
        }

        let (_, row) = cursor::position().map_err(terminal_error)?;
        let (_, terminal_rows) = terminal::size().map_err(terminal_error)?;
        let desired_start = row.saturating_add(1);
        let max_start = terminal_rows.saturating_sub(MODAL_HEIGHT);
        let start_row = desired_start.min(max_start);
        self.modal_start_row = Some(start_row);
        Ok(start_row)
    }

    fn clear_modal_area(
        &self,
        out: &mut impl Write,
        start_row: u16,
        height: u16,
    ) -> Result<(), ShowMeError> {
        for offset in 0..height {
            execute!(
                out,
                cursor::MoveTo(0, start_row.saturating_add(offset)),
                terminal::Clear(ClearType::CurrentLine)
            )
            .map_err(terminal_error)?;
        }
        Ok(())
    }

    fn filtered_entries(&self, category: &str) -> (Vec<CommandEntry>, bool) {
        let mut all = Vec::new();
        if let Some(operations) = self.catalog.ops.get(category) {
            for (operation, commands) in operations {
                for command in commands {
                    if command.command.trim().is_empty() {
                        continue;
                    }
                    all.push(CommandEntry {
                        operation: operation.clone(),
                        subsystem: command.subsystem.clone(),
                        command: command.command.clone(),
                    });
                }
            }
        }

        let subsystem = self.subsystem.as_str().to_lowercase();
        let filtered: Vec<CommandEntry> = all
            .iter()
            .filter(|entry| subsystem_matches(&entry.subsystem, &subsystem))
            .cloned()
            .collect();

        if filtered.is_empty() {
            (all, false)
        } else {
            (filtered, true)
        }
    }

    fn current_levels(&self, category: &str) -> String {
        self.catalog
            .levels
            .get(category)
            .filter(|levels| !levels.is_empty())
            .map(|levels| levels.join(", "))
            .unwrap_or_else(|| "General".to_owned())
    }

    fn confirm_execution(
        &mut self,
        out: &mut impl Write,
        command: &str,
    ) -> Result<bool, ShowMeError> {
        terminal::disable_raw_mode().map_err(terminal_error)?;
        execute!(out, cursor::Show, terminal::LeaveAlternateScreen).map_err(terminal_error)?;

        let prompt_result = (|| -> Result<bool, ShowMeError> {
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

        let resume_screen = execute!(
            out,
            terminal::EnterAlternateScreen,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide
        )
        .map_err(terminal_error);
        let resume_raw = terminal::enable_raw_mode().map_err(terminal_error);

        resume_screen?;
        resume_raw?;
        self.modal_start_row = None;
        prompt_result
    }
}

fn truncate(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_owned();
    }
    format!("{}...", &value[..max.saturating_sub(3)])
}

fn terminal_error(error: io::Error) -> ShowMeError {
    ShowMeError::Terminal(error.to_string())
}

fn subsystem_matches(entry_subsystem: &str, active_subsystem: &str) -> bool {
    if entry_subsystem.eq_ignore_ascii_case(active_subsystem) {
        return true;
    }

    if entry_subsystem.eq_ignore_ascii_case("bash/zsh") {
        return active_subsystem.eq_ignore_ascii_case("bash")
            || active_subsystem.eq_ignore_ascii_case("zsh");
    }

    false
}
