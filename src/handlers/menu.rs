use crate::utils::apply_visual_settings;
use geli_shell::{
    shell::{
        builtins::clear::clear_console_buffer,
        config::ShellConfig,
        reporter::Reporter,
        tui::{
            config_menu::{ConfigMenuSelection, show_config_menu_with_behavior},
            help_menu::{HelpMenuAction, show_help_menu},
            repl_input::SpecialCommand,
        },
    },
    t,
};

pub async fn handle_config_menu(config: &mut ShellConfig, reporter: &dyn Reporter) -> bool {
    match show_config_menu_with_behavior(&config.visual, config.behavior.reporter_level) {
        Ok(ConfigMenuSelection::Closed) => false,
        Ok(ConfigMenuSelection::UpdatedVisual(visual)) => {
            config.visual = visual;
            apply_visual_settings(config, reporter);

            if let Err(error) = config.save_async().await {
                reporter.error(&t!("config.menu_save_failed", error = error));
            } else {
                reporter.info(&t!("config.menu_updated"));
            }
            true
        }
        Ok(ConfigMenuSelection::UpdatedReporterLevel(level)) => {
            config.behavior.reporter_level = level;

            if let Err(error) = config.save_async().await {
                reporter.error(&t!("config.menu_save_failed", error = error));
            } else {
                reporter.info(&t!("config.menu_updated"));
            }
            true
        }
        Ok(ConfigMenuSelection::TomlEditor) => {
            reporter.warn(&t!("config.toml_warning"));
            let commands_path = ShellConfig::geli_config_dir().join("commands.toml");
            reporter.info(&t!(
                "config.customization_path",
                path = commands_path.display()
            ));
            false
        }
        Err(error) => {
            reporter.error(&t!("config.menu_failed", error = error));
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
            reporter.info(&t!("repl.goodbye"));
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
            reporter.error(&t!("help.menu_failed", error = error));
            false
        }
    }
}

pub fn handle_special_command(command: SpecialCommand, reporter: &dyn Reporter) {
    match command {
        SpecialCommand::Stop => {
            reporter.warn(&t!("special.stop_intercepted"));
        }
        SpecialCommand::Search => {
            reporter.info(&t!("special.search_skeleton"));
        }
    }
}

pub fn run_clear(config: &ShellConfig, reporter: &dyn Reporter) {
    if let Err(error) = clear_console_buffer() {
        reporter.error(&t!("exec.clear_failed", error = error));
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
