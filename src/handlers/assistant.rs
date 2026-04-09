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
    t,
};
use tokio::signal;

// These handlers implement the P2 assistant integration for the geli REPL.
// They are not yet wired to ReplInputAction::OpenAssistant — that wiring is
// deferred to P2 (see AGENTS.md § Pending Features).
#[allow(dead_code)]
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
            reporter.error(&t!("assistant.show_me_failed", error = e));
            return;
        }
    };

    let tokens = match Lexer::new(&selected_command).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => {
            reporter.error(&t!("assistant.show_me_invalid", error = error));
            return;
        }
    };

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => ast,
        Err(error) => {
            reporter.error(&t!("assistant.show_me_unparseable", error = error));
            return;
        }
    };

    if let Err(error) = guard.check(&ast) {
        reporter.error(&t!("assistant.show_me_blocked", error = error));
        return;
    }

    reporter.info(&t!(
        "assistant.show_me_executing",
        subsystem = subsystem.as_str(),
        command = selected_command
    ));

    tokio::select! {
        result = executor.run(&selected_command, exec_config, reporter) => {
            match result {
                Ok(exec_result) => {
                    builtins.record_g_visit();
                    if !exec_result.success() {
                        reporter.warn(&t!("assistant.show_me_exit_code", code = exec_result.exit_code));
                    }
                }
                Err(error) => reporter.error(&t!("assistant.show_me_exec_failed", error = error)),
            }
        }
        _ = signal::ctrl_c() => {
            println!();
            reporter.warn(&t!("assistant.show_me_ctrl_c"));
        }
    }
}

#[allow(dead_code)]
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
        reporter.error(&t!("assistant.bootstrap_ui_failed", error = error));
    }

    if let Err(error) = bootstrap_result {
        reporter.error(&t!("assistant.bootstrap_failed", error = error));
        assistant.release_resources().await;
        return;
    }

    let suggestion = match assistant.run_how_to(subsystem.as_str(), query).await {
        Ok(suggestion) => suggestion,
        Err(error) => {
            if let Err(ui_error) = show_assistant_error_panel(&error.to_string()) {
                reporter.error(&t!("assistant.error_panel_failed", ui_error = ui_error));
            }
            reporter.error(&t!("assistant.how_to_failed", error = error));
            assistant.release_resources().await;
            return;
        }
    };

    let should_execute =
        match show_how_to_confirmation_panel(&suggestion.explanation, &suggestion.command) {
            Ok(should_execute) => should_execute,
            Err(error) => {
                reporter.error(&t!("assistant.how_to_prompt_failed", error = error));
                assistant.release_resources().await;
                return;
            }
        };

    if !should_execute {
        reporter.info(&t!("assistant.how_to_cancelled"));
        assistant.release_resources().await;
        return;
    }

    let tokens = match Lexer::new(&suggestion.command).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => {
            reporter.error(&t!("assistant.generated_invalid", error = error));
            assistant.release_resources().await;
            return;
        }
    };

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => ast,
        Err(error) => {
            reporter.error(&t!("assistant.generated_unparseable", error = error));
            assistant.release_resources().await;
            return;
        }
    };

    if let Err(error) = guard.check(&ast) {
        reporter.error(&t!("assistant.command_blocked", error = error));
        assistant.release_resources().await;
        return;
    }

    reporter.info(&t!(
        "assistant.explanation",
        explanation = suggestion.explanation
    ));
    reporter.info(&t!(
        "assistant.executing",
        subsystem = subsystem.as_str(),
        command = suggestion.command
    ));

    tokio::select! {
        result = executor.run(&suggestion.command, exec_config, reporter) => {
            match result {
                Ok(exec_result) => {
                    builtins.record_g_visit();
                    if !exec_result.success() {
                        reporter.warn(&t!("assistant.command_exit_code", code = exec_result.exit_code));
                    }
                }
                Err(error) => reporter.error(&t!("assistant.command_exec_failed", error = error)),
            }
        }
        _ = signal::ctrl_c() => {
            println!();
            reporter.warn(&t!("assistant.ctrl_c"));
        }
    }

    assistant.release_resources().await;
}
