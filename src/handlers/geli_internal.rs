use crate::cli::execute_show_commands;
use crate::cli::print_cli_help;
use crate::handlers::menu::handle_config_menu;
use geli_shell::shell::{config::ShellConfig, reporter::Reporter};
use geli_shell::t;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeliInternalCommand {
    Help,
    ConfigMe,
    ShowCommands { ecosystem: String },
    SetLang(String),
    SetLangMissingArg,
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
        "lang" if parts.len() == 3 && parts[2] == "set" => {
            Some(GeliInternalCommand::SetLangMissingArg)
        }
        "lang" if parts.len() == 4 && parts[2] == "set" => {
            Some(GeliInternalCommand::SetLang(parts[3].to_owned()))
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
            reporter.warn(&t!("geli.no_args"));
        }
        GeliInternalCommand::Help => {
            print_cli_help();
        }
        GeliInternalCommand::ConfigMe => {
            if handle_config_menu(config, reporter).await {
                reporter.info(&t!("config.updated"));
            }
        }
        GeliInternalCommand::ShowCommands { ecosystem } => {
            let _ = execute_show_commands(&ecosystem, reporter).await;
        }
        GeliInternalCommand::SetLang(lang) => {
            let supported = geli_shell::shell::i18n::supported_languages();
            let normalized = lang.to_ascii_lowercase();
            if supported.contains(&normalized.as_str()) {
                geli_shell::shell::i18n::init_i18n(&normalized);
                reporter.info(&t!("geli.lang_set", lang = normalized));
            } else {
                reporter.warn(&t!("geli.lang_not_supported", lang = lang));
            }
        }
        GeliInternalCommand::SetLangMissingArg => {
            reporter.warn(&t!("geli.lang_set_missing_arg"));
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

    #[test]
    fn parses_lang_set_command() {
        let parsed = parse_geli_internal_command("geli lang set es");
        assert_eq!(parsed, Some(GeliInternalCommand::SetLang("es".to_owned())));
    }
}
