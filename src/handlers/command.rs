use crate::handlers::menu::handle_special_command;
use crate::utils::expand_custom_command;
use geli_shell::{
    t,
    parser::{ast::ASTNode, lexer::Lexer, parser::Parser},
    shell::{
        builtins::{BuiltinRegistry, BuiltinResult},
        config::{ShellConfig, SelectorMode},
        executor::{ExecutionConfig as ExecutorConfig, Executor},
        guard::Guard,
        reporter::Reporter,
        selector::{
            CommandSelector, SelectionResult,
            modal::ModalSelector,
        },
        translator::TranslationPipeline,
        tui::repl_input::{parse_special_command, SpecialCommand},
    },
};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::signal;

pub fn parse_ast(command: &str, reporter: &dyn Reporter) -> Option<ASTNode> {
    let tokens = match Lexer::new(command).tokenize() {
        Ok(t) => t,
        Err(e) => {
            reporter.error(&e.to_string());
            return None;
        }
    };
    match Parser::new(tokens).parse() {
        Ok(ast) => Some(ast),
        Err(e) => {
            reporter.error(&e.to_string());
            None
        }
    }
}

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

    let ast = match parse_ast(&expanded_input, reporter) {
        Some(ast) => ast,
        None => return false,
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

    let (command, resolved) = match pipeline.run_resolving(&ast, reporter) {
        Ok(pair) => pair,
        Err(error) => {
            reporter.error(&error.to_string());
            return false;
        }
    };

    let final_command = match config.behavior.selector_mode {
        SelectorMode::Always | SelectorMode::Once => {
            if let Some(res) = resolved.as_ref() {
                if res.has_alternatives() {
                    match ModalSelector::new().select(res) {
                        SelectionResult::Selected(chosen) => chosen,
                        SelectionResult::Cancelled => {
                            reporter.info(&t!("selector.cancelled"));
                            return false;
                        }
                    }
                } else {
                    command
                }
            } else {
                command
            }
        }
        SelectorMode::Auto => command,
    };

    execute_command_with_interrupts(
        &final_command,
        Executor::requires_tty(&final_command, &exec_config.extra_tty_commands),
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
                reporter.warn(&t!("exec.ctrl_c_interactive"));
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
            reporter.warn(&t!("exec.ctrl_c_stop"));
        }
        special = wait_for_runtime_special_command() => {
            println!();
            match special {
                SpecialCommand::Stop => reporter.warn(&t!("exec.stop_cancelled")),
                SpecialCommand::Search => reporter.warn(&t!("exec.search_cancelled")),
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
                reporter.warn(&t!("exec.exit_code", code = execution.exit_code));
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
