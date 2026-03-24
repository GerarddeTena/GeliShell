use geli_shell::{
    parser::{lexer::Lexer, parser::Parser},
    shell::{
        assistant::AssistantRuntime,
        builtins::BuiltinRegistry,
        config::{ShellConfig, VisualConfig},
        executor::{ExecutionConfig as ExecutorConfig, Executor},
        guard::Guard,
        reporter::Reporter,
        translator::Subsystem,
        tui::{
            assistant_menu::{
                show_assistant_error_panel, show_how_to_confirmation_panel,
                show_model_bootstrap_progress,
            },
            show_me::run_show_me_tui,
        },
    },
};
use tokio::signal;

pub async fn handle_assistant_show_me(
    subsystem: &Subsystem,
    guard: &dyn Guard,
    executor: &Executor,
    exec_config: &ExecutorConfig,
    builtins: &BuiltinRegistry,
    reporter: &dyn Reporter,
    visual: &VisualConfig,
) {
    let selected_command = match run_show_me_tui(reporter, visual) {
        Ok(Some(cmd)) => cmd,
        Ok(None) => return,
        Err(e) => {
            reporter.error(&format!("show-me failed: {e}"));
            return;
        }
    };

    let tokens = match Lexer::new(&selected_command).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => {
            reporter.error(&format!(
                "assistant --show-me generated invalid command: {error}"
            ));
            return;
        }
    };

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => ast,
        Err(error) => {
            reporter.error(&format!(
                "assistant --show-me generated unparseable command: {error}"
            ));
            return;
        }
    };

    if let Err(error) = guard.check(&ast) {
        reporter.error(&format!(
            "assistant --show-me command blocked by guard: {error}"
        ));
        return;
    }

    reporter.info(&format!(
        "assistant --show-me executing in {}: {}",
        subsystem.as_str(),
        selected_command
    ));

    tokio::select! {
        result = executor.run(&selected_command, exec_config, reporter) => {
            match result {
                Ok(exec_result) => {
                    builtins.record_g_visit();
                    if !exec_result.success() {
                        reporter.warn(&format!("assistant --show-me exit code: {}", exec_result.exit_code));
                    }
                }
                Err(error) => reporter.error(&format!("assistant --show-me execution failed: {error}")),
            }
        }
        _ = signal::ctrl_c() => {
            println!();
            reporter.warn("^C - assistant --show-me command cancelled");
        }
    }
}

pub async fn handle_assistant_how_to(
    assistant: &mut AssistantRuntime,
    config: &ShellConfig,
    subsystem: &Subsystem,
    guard: &dyn Guard,
    executor: &Executor,
    exec_config: &ExecutorConfig,
    builtins: &BuiltinRegistry,
    reporter: &dyn Reporter,
    query: &str,
) {
    assistant.refresh_config(config);

    let (progress_tx, progress_rx) = tokio::sync::mpsc::unbounded_channel();
    let bootstrap_future = assistant.ensure_model_ready(progress_tx);
    let progress_future = show_model_bootstrap_progress(progress_rx);
    let (bootstrap_result, progress_result) = tokio::join!(bootstrap_future, progress_future);

    if let Err(error) = progress_result {
        reporter.error(&format!("assistant bootstrap ui failed: {error}"));
    }

    if let Err(error) = bootstrap_result {
        reporter.error(&format!("assistant bootstrap failed: {error}"));
        assistant.release_resources().await;
        return;
    }

    let suggestion = match assistant.run_how_to(subsystem.as_str(), query).await {
        Ok(suggestion) => suggestion,
        Err(error) => {
            if let Err(ui_error) = show_assistant_error_panel(&error.to_string()) {
                reporter.error(&format!("assistant error panel failed: {ui_error}"));
            }
            reporter.error(&format!("assistant how-to failed: {error}"));
            assistant.release_resources().await;
            return;
        }
    };

    let should_execute =
        match show_how_to_confirmation_panel(&suggestion.explanation, &suggestion.command) {
            Ok(should_execute) => should_execute,
            Err(error) => {
                reporter.error(&format!("assistant how-to prompt failed: {error}"));
                assistant.release_resources().await;
                return;
            }
        };

    if !should_execute {
        reporter.info("assistant --how-to cancelled by user");
        assistant.release_resources().await;
        return;
    }

    let tokens = match Lexer::new(&suggestion.command).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => {
            reporter.error(&format!("assistant generated invalid command: {error}"));
            assistant.release_resources().await;
            return;
        }
    };

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => ast,
        Err(error) => {
            reporter.error(&format!("assistant generated unparseable command: {error}"));
            assistant.release_resources().await;
            return;
        }
    };

    if let Err(error) = guard.check(&ast) {
        reporter.error(&format!("assistant command blocked by guard: {error}"));
        assistant.release_resources().await;
        return;
    }

    reporter.info(&format!(
        "assistant explanation: {}",
        suggestion.explanation
    ));
    reporter.info(&format!(
        "assistant executing in {}: {}",
        subsystem.as_str(),
        suggestion.command
    ));

    tokio::select! {
        result = executor.run(&suggestion.command, exec_config, reporter) => {
            match result {
                Ok(exec_result) => {
                    builtins.record_g_visit();
                    if !exec_result.success() {
                        reporter.warn(&format!("assistant command exit code: {}", exec_result.exit_code));
                    }
                }
                Err(error) => reporter.error(&format!("assistant command execution failed: {error}")),
            }
        }
        _ = signal::ctrl_c() => {
            println!();
            reporter.warn("^C — assistant command cancelled");
        }
    }

    assistant.release_resources().await;
}

