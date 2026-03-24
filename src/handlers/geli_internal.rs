use crate::cli::print_cli_help;
use crate::cli::execute_show_commands;
use crate::handlers::menu::handle_config_menu;
use geli_shell::shell::{
    config::ShellConfig,
    reporter::Reporter,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeliInternalCommand {
    Help,
    ConfigMe,
    ShowCommands { ecosystem: String },
    NoArgs,
}

pub fn parse_geli_internal_command(input: &str) -> Option<GeliInternalCommand> {
    let trimmed = input.trim();

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() || parts[0] != "geli" {
        return None;
    }

    if parts.len() == 1 {
        return Some(GeliInternalCommand::NoArgs);
    }

    match parts[1] {
        "--help" | "-h" => Some(GeliInternalCommand::Help),
        "--config-me" => Some(GeliInternalCommand::ConfigMe),
        "--show" => {
            if parts.len() == 4 && parts[2] == "--commands" {
                Some(GeliInternalCommand::ShowCommands {
                    ecosystem: parts[3].to_owned(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

pub async fn handle_geli_internal_command(
    action: GeliInternalCommand,
    config: &mut ShellConfig,
    reporter: &dyn Reporter,
) {
    match action {
        GeliInternalCommand::NoArgs => {
            reporter.warn("Ya estás dentro de GeliShell. Usa 'exit' para salir o 'geli --help' para ver comandos disponibles.");
        }
        GeliInternalCommand::Help => {
            print_cli_help();
        }
        GeliInternalCommand::ConfigMe => {
            if handle_config_menu(config, reporter).await {
                reporter.info("config updated");
            }
        }
        GeliInternalCommand::ShowCommands { ecosystem } => {
            let _ = execute_show_commands(&ecosystem, reporter).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_show_commands_internal_invocation() {
        let parsed = parse_geli_internal_command("geli --show --commands git");
        assert_eq!(
            parsed,
            Some(GeliInternalCommand::ShowCommands {
                ecosystem: "git".to_owned()
            })
        );
    }

    #[test]
    fn rejects_show_without_commands_subflag() {
        let parsed = parse_geli_internal_command("geli --show git");
        assert_eq!(parsed, None);
    }
}

