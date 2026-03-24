use crate::handlers::assistant::{handle_assistant_how_to, handle_assistant_show_me};
use crate::handlers::menu::handle_config_menu;
use crate::utils::strip_wrapping_quotes;
use geli_shell::shell::{
    assistant::AssistantRuntime,
    builtins::BuiltinRegistry,
    config::ShellConfig,
    executor::{ExecutionConfig as ExecutorConfig, Executor},
    guard::Guard,
    reporter::Reporter,
    translator::Subsystem,
};

#[derive(Debug, Clone)]
pub enum GeliInternalCommand {
    Help,
    ConfigMe,
    HowTo { query: String },
    ShowMe,
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
        "--show-me" => Some(GeliInternalCommand::ShowMe),
        "--how-to" => {
            if parts.len() > 2 {
                let query = parts[2..].join(" ");
                let query = strip_wrapping_quotes(&query);
                Some(GeliInternalCommand::HowTo {
                    query: query.to_owned(),
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
    assistant: &mut AssistantRuntime,
    subsystem: &Subsystem,
    guard: &dyn Guard,
    executor: &Executor,
    exec_config: &ExecutorConfig,
    builtins: &BuiltinRegistry,
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
        GeliInternalCommand::HowTo { query } => {
            if query.trim().is_empty() {
                reporter.error("--how-to requires a non-empty query");
                return;
            }
            handle_assistant_how_to(
                assistant,
                config,
                subsystem,
                guard,
                executor,
                exec_config,
                builtins,
                reporter,
                &query,
            )
            .await;
        }
        GeliInternalCommand::ShowMe => {
            handle_assistant_show_me(
                subsystem,
                guard,
                executor,
                exec_config,
                builtins,
                reporter,
                &config.visual,
            )
            .await;
        }
    }
}

fn print_cli_help() {
    println!("GeliShell 0.1.0");
    println!();
    println!("USAGE:");
    println!("    geli [FLAGS]");
    println!();
    println!("FLAGS:");
    println!("    --help, -h           Show this help message");
    println!("    --config-me          Open configuration menu");
    println!("    --how-to <query>     Ask the assistant how to do something");
    println!("    --show-me            Interactive command search with assistant");
    println!();
    println!("If no flags are provided, GeliShell will start in interactive mode.");
}
