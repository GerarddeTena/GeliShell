use crate::handlers::menu::handle_special_command;
use crate::utils::expand_custom_command;
use geli_shell::{
    parser::{lexer::Lexer, parser::Parser},
    shell::{
        builtins::{BuiltinRegistry, BuiltinResult},
        config::ShellConfig,
        executor::{ExecutionConfig as ExecutorConfig, Executor},
        guard::Guard,
        reporter::{Reporter, SilentReporter},
        translator::TranslationPipeline,
        tui::repl_input::{parse_special_command, SpecialCommand},
    },
};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::signal;

pub async fn process_regular_command(
    input: &str,
    config: &ShellConfig,
    guard: &dyn Guard,
    pipeline: &TranslationPipeline,
    executor: &Executor,
    exec_config: &ExecutorConfig,
    builtins: &mut BuiltinRegistry,
    reporter: &dyn Reporter,
) -> bool {
    builtins.push_history(input.to_owned());

    let expanded_input = expand_custom_command(input, config);

    let tokens = match Lexer::new(&expanded_input).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => {
            reporter.error(&error.to_string());
            return false;
        }
    };
    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => ast,
        Err(error) => {
            reporter.error(&error.to_string());
            return false;
        }
    };

    match builtins.try_execute(&ast, reporter) {
        BuiltinResult::Handled => {
            builtins.record_g_visit();
            if expanded_input.trim() == "clear" {
                crate::utils::apply_visual_settings(config, reporter);
            }
            return false;
        }
        BuiltinResult::Exit(code) => std::process::exit(code),
        BuiltinResult::NotABuiltin => {}
    }

    if let Err(error) = guard.check(&ast) {
        reporter.error(&error.to_string());
        return false;
    }

    let command = match pipeline.run(&ast, &SilentReporter::new()) {
        Ok(command) => command,
        Err(error) => {
            reporter.error(&error.to_string());
            return false;
        }
    };

    let command_str = command.clone();
    let _final_command = match config.behavior.selector_mode {
        geli_shell::shell::config::SelectorMode::Auto => command,
        geli_shell::shell::config::SelectorMode::Always
        | geli_shell::shell::config::SelectorMode::Once => command,
    };

    execute_command_with_interrupts(
        &command_str,
        Executor::requires_tty(&command_str),
        executor,
        exec_config,
        builtins,
        reporter,
    )
    .await;

    true
}

pub async fn execute_command_with_interrupts(
    command: &str,
    interactive_tty: bool,
    executor: &Executor,
    exec_config: &ExecutorConfig,
    builtins: &BuiltinRegistry,
    reporter: &dyn Reporter,
) {
    if interactive_tty {
        tokio::select! {
            result = executor.run(command, exec_config, reporter) => {
                report_executor_outcome(result, builtins, reporter);
            }
            _ = signal::ctrl_c() => {
                println!();
                reporter.warn("^C — command cancelled");
            }
        }
        return;
    }

    tokio::select! {
        result = executor.run(command, exec_config, reporter) => {
            report_executor_outcome(result, builtins, reporter);
        }
        _ = signal::ctrl_c() => {
            println!();
            reporter.warn("^C/:stop* — command cancelled");
        }
        special = wait_for_runtime_special_command() => {
            println!();
            match special {
                SpecialCommand::Stop => reporter.warn(":stop* — command cancelled"),
                SpecialCommand::Search => reporter.warn(":search* — command cancelled"),
            }
            handle_special_command(special, reporter);
        }
    }
}

pub fn report_executor_outcome(
    result: Result<geli_shell::ExecutionResult, geli_shell::shell::executor::ExecutorError>,
    builtins: &BuiltinRegistry,
    reporter: &dyn Reporter,
) {
    match result {
        Ok(execution) => {
            builtins.record_g_visit();

            if !execution.success() {
                reporter.warn(&format!("exit code: {}", execution.exit_code));
            }
        }
        Err(error) => {
            reporter.error(&error.to_string());
            let _ = std::io::Write::flush(&mut std::io::stderr());
        }
    }
}

pub async fn wait_for_runtime_special_command() -> SpecialCommand {
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    loop {
        line.clear();
        match tokio::time::timeout(
            std::time::Duration::from_millis(50),
            reader.read_line(&mut line),
        )
        .await
        {
            Ok(Ok(0)) | Ok(Err(_)) | Err(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            Ok(Ok(_)) => {
                if let Some(command) = parse_special_command(line.trim()) {
                    return command;
                }
            }
        }
    }
}

pub fn drain_crossterm_events(_reporter: &dyn Reporter) {
    use crossterm::event::{poll, read};

    for _ in 0..50 {
        match poll(Duration::from_millis(0)) {
            Ok(true) => {
                if read().is_err() {
                    break;
                }
            }
            Ok(false) => break,
            Err(_) => break,
        }
    }

    std::thread::sleep(Duration::from_millis(10));
}
