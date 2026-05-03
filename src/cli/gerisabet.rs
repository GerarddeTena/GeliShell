use crate::handlers::assistant::{handle_assistant_how_to, handle_assistant_show_me};
use geli_shell::shell::{
    assistant::AssistantRuntime, builtins::BuiltinRegistry, config::ShellConfig,
    executor::Executor, guard::default_guard_normalized, reporter::Reporter, translator::load,
};

pub async fn handle_gerisabet_args(args: &[String], reporter: &dyn Reporter) {
    if args.is_empty() {
        print_gerisabet_help();
        std::process::exit(0);
    }

    match args[0].as_str() {
        "--help" | "-h" => {
            print_gerisabet_help();
            std::process::exit(0);
        }
        "--how-to" => {
            if args.len() < 2 {
                reporter.error("--how-to requires a query argument");
                reporter.info("Usage: gerisabet --how-to \"<query>\"");
                std::process::exit(1);
            }

            let query = args[1..].join(" ");
            let query = strip_wrapping_quotes(&query);
            if query.trim().is_empty() {
                reporter.error("--how-to query cannot be empty");
                std::process::exit(1);
            }

            execute_how_to_cli(query, reporter).await;
            std::process::exit(0);
        }
        "--show-me" => {
            if args.len() > 1 {
                reporter.error("--show-me does not accept additional arguments");
                std::process::exit(1);
            }

            execute_show_me_cli(reporter).await;
            std::process::exit(0);
        }
        unknown => {
            reporter.error(&format!("unknown gerisabet flag '{unknown}'"));
            print_gerisabet_help();
            std::process::exit(1);
        }
    }
}

pub fn print_gerisabet_help() {
    println!("Gerisabet 0.1.0 - GeliShell AI Assistant");
    println!();
    println!("USAGE:");
    println!("    gerisabet [FLAGS]");
    println!();
    println!("FLAGS:");
    println!("    --help, -h           Show this help message");
    println!("    --how-to <query>     Ask the assistant how to do something");
    println!("    --show-me            Interactive command search with assistant");
    println!();
    println!("Examples:");
    println!("    gerisabet --how-to \"compress a folder\"");
    println!("    gerisabet --show-me");
}

async fn execute_how_to_cli(query: &str, reporter: &dyn Reporter) {
    let config = ShellConfig::load_async().await.unwrap_or_default();

    let subsystem = config.resolve_subsystem(reporter);
    let map = match load() {
        Ok(result) => std::sync::Arc::new(result.map),
        Err(error) => {
            reporter.error(&error.to_string());
            return;
        }
    };
    let guard = Box::new(default_guard_normalized(map));
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
    let config = ShellConfig::load_async().await.unwrap_or_default();

    let subsystem = config.resolve_subsystem(reporter);
    let map = match load() {
        Ok(result) => std::sync::Arc::new(result.map),
        Err(error) => {
            reporter.error(&error.to_string());
            return;
        }
    };
    let guard = Box::new(default_guard_normalized(map));
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

fn strip_wrapping_quotes(input: &str) -> &str {
    if input.len() < 2 {
        return input;
    }

    let bytes = input.as_bytes();
    let starts_with_double = bytes.first() == Some(&b'"');
    let ends_with_double = bytes.last() == Some(&b'"');
    let starts_with_single = bytes.first() == Some(&b'\'');
    let ends_with_single = bytes.last() == Some(&b'\'');

    if (starts_with_double && ends_with_double) || (starts_with_single && ends_with_single) {
        &input[1..input.len() - 1]
    } else {
        input
    }
}
