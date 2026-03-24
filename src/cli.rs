use crate::handlers::assistant::{handle_assistant_how_to, handle_assistant_show_me};
use crate::handlers::menu::handle_config_menu;
use crate::setup::resolve_subsystem;
use crate::utils::strip_wrapping_quotes;
use geli_shell::shell::{
    assistant::AssistantRuntime,
    builtins::BuiltinRegistry,
    config::ShellConfig,
    executor::Executor,
    guard::default_guard,
    reporter::{Reporter, StderrReporter},
};

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
            let mut config = match ShellConfig::load_async().await {
                Ok(cfg) => cfg,
                Err(_) => ShellConfig::default(),
            };

            if handle_config_menu(&mut config, &reporter).await {
                reporter.info("config updated");
            }
            std::process::exit(0);
        }
        "--how-to" => {
            if args.len() < 2 {
                eprintln!("Error: --how-to requires a query argument");
                eprintln!("Usage: geli --how-to \"<query>\"");
                std::process::exit(1);
            }
            let query = args[1..].join(" ");
            let query = strip_wrapping_quotes(&query);

            if query.trim().is_empty() {
                eprintln!("Error: --how-to query cannot be empty");
                std::process::exit(1);
            }

            reporter.info("initializing assistant for --how-to...");
            execute_how_to_cli(&query, &reporter).await;
            std::process::exit(0);
        }
        "--show-me" => {
            if args.len() > 1 {
                eprintln!("Error: --show-me does not accept additional arguments");
                std::process::exit(1);
            }

            reporter.info("initializing assistant for --show-me...");
            execute_show_me_cli(&reporter).await;
            std::process::exit(0);
        }
        unknown => {
            eprintln!("Error: unknown flag '{unknown}'");
            eprintln!();
            print_cli_help();
            std::process::exit(1);
        }
    }
}

pub fn print_cli_help() {
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

async fn execute_how_to_cli(query: &str, reporter: &dyn Reporter) {
    let config = match ShellConfig::load_async().await {
        Ok(cfg) => cfg,
        Err(_) => ShellConfig::default(),
    };

    let subsystem = resolve_subsystem(&config, reporter);
    let guard = Box::new(default_guard());
    let executor = Executor::new(subsystem.clone());
    let exec_config = config.to_executor_config();
    let builtins = BuiltinRegistry::new();
    let mut assistant = AssistantRuntime::new(&config);

    handle_assistant_how_to(
        &mut assistant,
        &config,
        &subsystem,
        guard.as_ref(),
        &executor,
        &exec_config,
        &builtins,
        reporter,
        query,
    )
    .await;
}

async fn execute_show_me_cli(reporter: &dyn Reporter) {
    let config = match ShellConfig::load_async().await {
        Ok(cfg) => cfg,
        Err(_) => ShellConfig::default(),
    };

    let subsystem = resolve_subsystem(&config, reporter);
    let guard = Box::new(default_guard());
    let executor = Executor::new(subsystem.clone());
    let exec_config = config.to_executor_config();
    let builtins = BuiltinRegistry::new();

    handle_assistant_show_me(
        &subsystem,
        guard.as_ref(),
        &executor,
        &exec_config,
        &builtins,
        reporter,
        &config.visual,
    )
    .await;
}
