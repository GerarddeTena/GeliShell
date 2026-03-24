use crate::handlers::assistant::{handle_assistant, handle_assistant_how_to, handle_assistant_show_me, parse_assistant_invocation, AssistantInvocation};
use crate::handlers::command::{drain_crossterm_events, process_regular_command};
use crate::handlers::geli_internal::{handle_geli_internal_command, parse_geli_internal_command};
use crate::handlers::menu::{handle_config_menu, handle_help_menu, handle_special_command, is_config_trigger, is_help_trigger, run_clear};
use crate::utils::{append_history_or_warn, build_completion_pool, render_prompt};
use geli_shell::shell::{
    assistant::AssistantRuntime,
    builtins::BuiltinRegistry,
    config::{history_store::PersistentCommandHistory, ShellConfig},
    executor::{ExecutionConfig as ExecutorConfig, Executor},
    guard::Guard,
    reporter::Reporter,
    translator::{CommandMap, Subsystem, TranslationPipeline},
    tui::repl_input::{parse_special_command, read_repl_input, ReplInputAction, SpecialCommand},
};

#[allow(clippy::too_many_arguments)]
pub async fn run_repl(
    mut config: ShellConfig,
    mut command_history: PersistentCommandHistory,
    map: std::sync::Arc<CommandMap>,
    subsystem: Subsystem,
    pipeline: TranslationPipeline,
    executor: Executor,
    exec_config: ExecutorConfig,
    guard: Box<dyn Guard>,
    mut builtins: BuiltinRegistry,
    mut assistant: AssistantRuntime,
    reporter: &dyn Reporter,
) {
    let mut completion_pool = build_completion_pool(map.as_ref(), &config);

    loop {
        assistant.sweep_idle_resources();
        let g_jump_paths = builtins.g_completion_paths(64);
        let prompt = render_prompt(&subsystem, &config.visual);
        let input = match read_repl_input(
            &prompt,
            command_history.entries(),
            &completion_pool,
            &g_jump_paths,
            config.visual.prompt_dim_ansi256,
        ) {
            Ok(ReplInputAction::Command(line)) => line,
            Ok(ReplInputAction::Exit) => {
                reporter.info("goodbye");
                break;
            }
            Ok(ReplInputAction::OpenHelp) => {
                if handle_help_menu(&config, reporter) {
                    break;
                }
                continue;
            }
            Ok(ReplInputAction::OpenConfig) => {
                if handle_config_menu(&mut config, reporter).await {
                    completion_pool = build_completion_pool(map.as_ref(), &config);
                    assistant.refresh_config(&config);
                }
                continue;
            }
            Ok(ReplInputAction::OpenAssistant) => {
                handle_assistant(&mut assistant, &config, reporter).await;
                continue;
            }
            Ok(ReplInputAction::Clear) => {
                run_clear(&config, reporter);
                continue;
            }
            Ok(ReplInputAction::Search) => {
                handle_special_command(SpecialCommand::Search, reporter);
                continue;
            }
            Err(error) => {
                reporter.error(&format!("input error: {error}"));
                break;
            }
        };

        if input.is_empty() {
            continue;
        }

        if is_help_trigger(&input) {
            append_history_or_warn(&mut command_history, &input, reporter).await;
            if handle_help_menu(&config, reporter) {
                break;
            }
            continue;
        }

        if is_config_trigger(&input) {
            append_history_or_warn(&mut command_history, &input, reporter).await;

            if input.eq_ignore_ascii_case("geli-reset-config") {
                match ShellConfig::reset().await {
                    Ok(()) => {
                        reporter.info("config reset — restart GeliShell to run the setup wizard");
                    }
                    Err(error) => {
                        reporter.error(&format!("reset failed: {error}"));
                    }
                }
            } else if handle_config_menu(&mut config, reporter).await {
                completion_pool = build_completion_pool(map.as_ref(), &config);
                assistant.refresh_config(&config);
            }
            continue;
        }
        match parse_assistant_invocation(&input) {
            Ok(Some(AssistantInvocation::Menu)) => {
                append_history_or_warn(&mut command_history, &input, reporter).await;
                handle_assistant(&mut assistant, &config, reporter).await;
                continue;
            }
            Ok(Some(AssistantInvocation::HowTo { query })) => {
                append_history_or_warn(&mut command_history, &input, reporter).await;
                handle_assistant_how_to(
                    &mut assistant,
                    &config,
                    &subsystem,
                    guard.as_ref(),
                    &executor,
                    &exec_config,
                    &builtins,
                    reporter,
                    &query,
                )
                .await;
                continue;
            }
            Ok(Some(AssistantInvocation::ShowMe)) => {
                append_history_or_warn(&mut command_history, &input, reporter).await;
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
                continue;
            }
            Ok(None) => {}
            Err(error) => {
                reporter.error(&error);
                continue;
            }
        }

        if let Some(special) = parse_special_command(&input) {
            handle_special_command(special, reporter);
            continue;
        }

        if let Some(action) = parse_geli_internal_command(&input) {
            append_history_or_warn(&mut command_history, &input, reporter).await;
            handle_geli_internal_command(
                action,
                &mut config,
                &mut assistant,
                &subsystem,
                guard.as_ref(),
                &executor,
                &exec_config,
                &builtins,
                reporter,
            )
            .await;
            continue;
        }

        append_history_or_warn(&mut command_history, &input, reporter).await;

        if process_regular_command(
            &input,
            &config,
            guard.as_ref(),
            &pipeline,
            &executor,
            &exec_config,
            &mut builtins,
            reporter,
        )
        .await
        {
            drain_crossterm_events(reporter);
        }
    }
}
