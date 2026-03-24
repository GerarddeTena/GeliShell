use crate::handlers::menu::handle_config_menu;
use crate::setup::resolve_subsystem;
use geli_shell::{
    parser::{lexer::Lexer, parser::Parser},
    shell::{
    commands::ecosystems::registry::EcosystemRegistry,
    builtins::BuiltinRegistry,
    config::ShellConfig,
    executor::Executor,
    guard::{Guard, default_guard},
    reporter::{Reporter, StderrReporter},
    tui::ecosystem::EcosystemTui,
    },
};
use tokio::signal;


pub async fn handle_cli_args(args: &[String]) {
    let reporter = StderrReporter::new();

    if args.is_empty() {
        return;
    }

    let first = args[0].as_str();

    match first {
        "--help" | "-h" => {
            print_cli_help();
            std::process::exit(0);
        }
        "--config-me" => {
            let mut config = ShellConfig::load_async().await.unwrap_or_default();

            if handle_config_menu(&mut config, &reporter).await {
                reporter.info("config updated");
            }
            std::process::exit(0);
        }
        "--show" => {
            match run_show_commands_args(args, &reporter).await {
                Ok(()) => std::process::exit(0),
                Err(_) => std::process::exit(1),
            }
        }
        unknown => {
            reporter.error(&format!("unknown flag '{unknown}'"));
            print_cli_help();
            std::process::exit(1);
        }
    }
}

pub async fn run_show_commands_args(args: &[String], reporter: &dyn Reporter) -> Result<(), ()> {
    if args.get(1).map(String::as_str) != Some("--commands") {
        reporter.error("--show requires --commands <ecosystem>");
        reporter.info("Usage: geli --show --commands <ecosystem>");
        print_cli_help();
        return Err(());
    }

    let ecosystem = match args.get(2) {
        Some(name) => name.as_str(),
        None => {
            reporter.error("missing ecosystem name");
            reporter.info(&format!(
                "Available: {}",
                EcosystemRegistry::available().join(", ")
            ));
            return Err(());
        }
    };

    execute_show_commands(ecosystem, reporter).await
}

pub fn print_cli_help() {
    println!("GeliShell 0.1.0");
    println!();
    println!("USAGE:");
    println!("    geli [FLAGS]");
    println!();
    println!("FLAGS:");
    println!("    --help, -h                    Show this help message");
    println!("    --config-me                   Open configuration menu");
    println!("    --show --commands <ecosystem> Browse ecosystem commands (TUI)");
    println!("                                  Available: cargo, docker, dotnet, git, npm, python");
    println!();
    println!("AI ASSISTANT:");
    println!("    gerisabet --how-to <query>    Ask the assistant how to do something");
    println!("    gerisabet --show-me           Interactive command search");
    println!();
    println!("If no flags are provided, GeliShell starts in interactive mode.");
}

pub async fn execute_show_commands(ecosystem: &str, reporter: &dyn Reporter) -> Result<(), ()> {
    let config = ShellConfig::load_async().await.unwrap_or_default();

    let subsystem = resolve_subsystem(&config, reporter);
    let executor = Executor::new(subsystem.clone());
    let exec_config = config.to_executor_config();
    let guard = Box::new(default_guard());
    let builtins = BuiltinRegistry::new();

    let registry = match EcosystemRegistry::load() {
        Ok(registry) => registry,
        Err(error) => {
            reporter.error(&error.to_string());
            return Err(());
        }
    };

    let catalog = match registry.get(ecosystem) {
        Some(catalog) => catalog.clone(),
        None => {
            reporter.error(&format!(
                "unknown ecosystem '{}'. Available: {}",
                ecosystem,
                EcosystemRegistry::available().join(", ")
            ));
            return Err(());
        }
    };

    let selected = match EcosystemTui::new(catalog, subsystem.clone()).run(reporter).await {
        Ok(selected) => selected,
        Err(error) => {
            reporter.error(&format!("ecosystem tui failed: {error}"));
            return Err(());
        }
    };
    let Some(command) = selected else {
        return Ok(());
    };

    let tokens = match Lexer::new(&command).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => {
            reporter.error(&format!("ecosystem command is invalid: {error}"));
            return Err(());
        }
    };

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => ast,
        Err(error) => {
            reporter.error(&format!("ecosystem command is unparseable: {error}"));
            return Err(());
        }
    };

    if let Err(error) = guard.check(&ast) {
        reporter.error(&format!("ecosystem command blocked by guard: {error}"));
        return Err(());
    }

    reporter.info(&format!(
        "ecosystem command executing in {}: {}",
        subsystem.as_str(),
        command
    ));

    tokio::select! {
        result = executor.run(&command, &exec_config, reporter) => {
            match result {
                Ok(exec_result) => {
                    builtins.record_g_visit();
                    if !exec_result.success() {
                        reporter.warn(&format!("ecosystem command exit code: {}", exec_result.exit_code));
                    }
                }
                Err(error) => reporter.error(&format!("ecosystem command execution failed: {error}")),
            }
        }
        _ = signal::ctrl_c() => {
            reporter.warn("^C - ecosystem command cancelled");
        }
    }

    Ok(())
}
