use crate::handlers::menu::handle_config_menu;
use crate::setup::resolve_subsystem;
use geli_shell::{
    parser::{lexer::Lexer, parser::Parser},
    shell::{
        builtins::BuiltinRegistry,
        commands::ecosystems::registry::EcosystemRegistry,
        config::ShellConfig,
        executor::Executor,
        guard::{Guard, default_guard_normalized},
        reporter::{Reporter, StderrReporter},
        tui::ecosystem::EcosystemTui,
        translator::load,
    },
    t,
};
use tokio::signal;

pub async fn handle_cli_args(args: &[String]) {
    let config = ShellConfig::load_async().await.unwrap_or_default();
    let reporter = StderrReporter::new(config.behavior.reporter_level);

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
            let mut config = config;

            if handle_config_menu(&mut config, &reporter).await {
                reporter.info(&t!("cli.config_updated"));
            }
            std::process::exit(0);
        }
        "--show" => match run_show_commands_args(args, &reporter).await {
            Ok(()) => std::process::exit(0),
            Err(_) => std::process::exit(1),
        },
        unknown => {
            reporter.error(&t!("cli.unknown_flag", flag = unknown));
            print_cli_help();
            std::process::exit(1);
        }
    }
}

pub async fn run_show_commands_args(args: &[String], reporter: &dyn Reporter) -> Result<(), ()> {
    if args.get(1).map(String::as_str) != Some("--commands") {
        reporter.error(&t!("cli.show_requires_commands"));
        reporter.info(&t!("cli.show_usage"));
        print_cli_help();
        return Err(());
    }

    let ecosystem = match args.get(2) {
        Some(name) => name.as_str(),
        None => {
            reporter.error(&t!("cli.missing_ecosystem"));
            reporter.info(&t!(
                "cli.available_ecosystems",
                list = EcosystemRegistry::available().join(", ")
            ));
            return Err(());
        }
    };

    execute_show_commands(ecosystem, reporter).await
}

pub fn print_cli_help() {
    println!("{}", t!("cli.help.title"));
    println!();
    println!("{}", t!("cli.help.usage_header"));
    println!("{}", t!("cli.help.usage"));
    println!();
    println!("{}", t!("cli.help.flags_header"));
    println!("{}", t!("cli.help.flag_help"));
    println!("{}", t!("cli.help.flag_config"));
    println!("{}", t!("cli.help.flag_show"));
    println!("{}", t!("cli.help.flag_show_available"));
    println!();
    println!("{}", t!("cli.help.ai_header"));
    println!("{}", t!("cli.help.ai_how_to"));
    println!("{}", t!("cli.help.ai_show_me"));
    println!();
    println!("{}", t!("cli.help.footer"));
}

pub async fn execute_show_commands(ecosystem: &str, reporter: &dyn Reporter) -> Result<(), ()> {
    let config = ShellConfig::load_async().await.unwrap_or_default();

    let subsystem = resolve_subsystem(&config, reporter);
    let executor = Executor::new(subsystem.clone());
    let exec_config = config.to_executor_config();
    let map = match load() {
        Ok(result) => std::sync::Arc::new(result.map),
        Err(error) => {
            reporter.error(&error.to_string());
            return Err(());
        }
    };
    let guard = Box::new(default_guard_normalized(map));
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
            reporter.error(&t!(
                "cli.unknown_ecosystem",
                name = ecosystem,
                list = EcosystemRegistry::available().join(", ")
            ));
            return Err(());
        }
    };

    let selected = match EcosystemTui::new(catalog, subsystem.clone())
        .run(reporter)
        .await
    {
        Ok(selected) => selected,
        Err(error) => {
            reporter.error(&t!("cli.tui_failed", error = error));
            return Err(());
        }
    };
    let Some(command) = selected else {
        return Ok(());
    };

    let tokens = match Lexer::new(&command).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => {
            reporter.error(&t!("cli.command_invalid", error = error));
            return Err(());
        }
    };

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => ast,
        Err(error) => {
            reporter.error(&t!("cli.command_unparseable", error = error));
            return Err(());
        }
    };

    if let Err(error) = guard.check(&ast) {
        reporter.error(&t!("cli.command_blocked", error = error));
        return Err(());
    }

    reporter.info(&t!(
        "cli.command_executing",
        subsystem = subsystem.as_str(),
        command = command
    ));

    tokio::select! {
        result = executor.run(&command, &exec_config, reporter) => {
            match result {
                Ok(exec_result) => {
                    builtins.record_g_visit();
                    if !exec_result.success() {
                        reporter.warn(&t!("cli.command_exit_code", code = exec_result.exit_code));
                    }
                }
                Err(error) => reporter.error(&t!("cli.command_exec_failed", error = error)),
            }
        }
        _ = signal::ctrl_c() => {
            reporter.warn(&t!("cli.ctrl_c"));
        }
    }

    Ok(())
}
