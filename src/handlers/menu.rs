use crate::utils::apply_visual_settings;
use geli_shell::shell::{
    builtins::clear::clear_console_buffer,
    config::ShellConfig,
    reporter::Reporter,
    tui::{
        config_menu::{show_config_menu, ConfigMenuSelection},
        help_menu::{show_help_menu, HelpMenuAction},
        repl_input::SpecialCommand,
    },
};

pub async fn handle_config_menu(config: &mut ShellConfig, reporter: &dyn Reporter) -> bool {
    match show_config_menu(&config.visual) {
        Ok(ConfigMenuSelection::Closed) => false,
        Ok(ConfigMenuSelection::UpdatedVisual(visual)) => {
            config.visual = visual;
            apply_visual_settings(config, reporter);

            if let Err(error) = config.save_async().await {
                reporter.error(&format!("config save failed: {error}"));
            } else {
                reporter.info("config updated and persisted");
            }
            true
        }
        Ok(ConfigMenuSelection::TomlEditor) => {
            reporter.warn("WARNING: editing command toml can break the shell if invalid");
            let commands_path = std::env::current_dir()
                .unwrap_or_default()
                .join("src")
                .join("commands")
                .join("commands.toml");
            reporter.info(&format!("customization file: {}", commands_path.display()));
            false
        }
        Err(error) => {
            reporter.error(&format!("config menu failed: {error}"));
            false
        }
    }
}

pub fn handle_help_menu(config: &ShellConfig, reporter: &dyn Reporter) -> bool {
    match show_help_menu() {
        Ok(HelpMenuAction::None) => false,
        Ok(HelpMenuAction::Clear) => {
            run_clear(config, reporter);
            false
        }
        Ok(HelpMenuAction::Exit) => {
            reporter.info("goodbye");
            true
        }
        Ok(HelpMenuAction::Stop) => {
            handle_special_command(SpecialCommand::Stop, reporter);
            false
        }
        Ok(HelpMenuAction::Search) => {
            handle_special_command(SpecialCommand::Search, reporter);
            false
        }
        Err(error) => {
            reporter.error(&format!("help menu failed: {error}"));
            false
        }
    }
}

pub fn handle_special_command(command: SpecialCommand, reporter: &dyn Reporter) {
    match command {
        SpecialCommand::Stop => {
            reporter.warn(":stop* intercepted — use Ctrl+C to interrupt running command");
        }
        SpecialCommand::Search => {
            reporter.info(":search* intercepted — interactive search UI is active as skeleton");
        }
    }
}

pub fn run_clear(config: &ShellConfig, reporter: &dyn Reporter) {
    if let Err(error) = clear_console_buffer() {
        reporter.error(&format!("clear failed: {error}"));
        return;
    }
    apply_visual_settings(config, reporter);
}

pub fn is_help_trigger(input: &str) -> bool {
    matches!(input, "^?" | "^H" | "\u{8}" | "\u{7f}")
}

pub fn is_config_trigger(input: &str) -> bool {
    input.eq_ignore_ascii_case("geli-reset-config")
}
