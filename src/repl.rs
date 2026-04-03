use crate::handlers::command::{drain_crossterm_events, process_regular_command};
use crate::handlers::geli_internal::{handle_geli_internal_command, parse_geli_internal_command};
use crate::handlers::menu::{handle_config_menu, handle_help_menu, handle_special_command, is_config_trigger, is_help_trigger, run_clear};
use crate::utils::{append_history_or_warn, build_completion_pool, render_prompt};
use geli_shell::t;
use geli_shell::shell::{
    builtins::BuiltinRegistry,
    config::{history_store::PersistentCommandHistory, ShellConfig},
    executor::{ExecutionConfig as ExecutorConfig, Executor},
    guard::Guard,
    reporter::Reporter,
    translator::{CommandMap, Subsystem, TranslationPipeline},
    tui::repl_input::{parse_special_command, read_repl_input, ReplInputAction, SpecialCommand},
};
use std::collections::HashSet;
use std::sync::Arc;

// ══════════════════════════════════════════════════════════════
// ReplContext — agrupa los parámetros de larga vida del REPL
// ══════════════════════════════════════════════════════════════

pub struct ReplContext {
    pub config: ShellConfig,
    pub command_history: PersistentCommandHistory,
    pub map: Arc<CommandMap>,
    pub subsystem: Subsystem,
    pub pipeline: TranslationPipeline,
    pub executor: Executor,
    pub exec_config: ExecutorConfig,
    pub guard: Box<dyn Guard>,
    pub builtins: BuiltinRegistry,
}

pub async fn run_repl(mut ctx: ReplContext, reporter: &dyn Reporter) {
    let mut completion_pool = build_completion_pool(ctx.map.as_ref(), &ctx.config, &ctx.subsystem);
    // Tracks which commands have already shown the ModalSelector this session.
    // Used by SelectorMode::Once to avoid re-interrupting known commands.
    let mut seen_once: HashSet<String> = HashSet::new();

    loop {
        let g_jump_paths = ctx.builtins.g_completion_paths(64);
        let prompt = render_prompt(&ctx.subsystem, &ctx.config.visual);
        let input = match read_repl_input(
            &prompt,
            ctx.command_history.entries(),
            &completion_pool,
            &g_jump_paths,
            ctx.config.visual.prompt_dim_ansi256,
        ) {
            Ok(ReplInputAction::Command(line)) => line,
            Ok(ReplInputAction::Exit) => {
                reporter.info(&t!("repl.goodbye"));
                break;
            }
            Ok(ReplInputAction::OpenHelp) => {
                if handle_help_menu(&ctx.config, reporter) {
                    break;
                }
                continue;
            }
            Ok(ReplInputAction::OpenConfig) => {
                if handle_config_menu(&mut ctx.config, reporter).await {
                    completion_pool = build_completion_pool(ctx.map.as_ref(), &ctx.config, &ctx.subsystem);
                }
                continue;
            }
            Ok(ReplInputAction::OpenAssistant) => {
                reporter.warn(&t!("repl.assistant_moved"));
                reporter.info(&t!("repl.assistant_hint"));
                continue;
            }
            Ok(ReplInputAction::Clear) => {
                run_clear(&ctx.config, reporter);
                continue;
            }
            Ok(ReplInputAction::Search) => {
                handle_special_command(SpecialCommand::Search, reporter);
                continue;
            }
            Err(error) => {
                reporter.error(&t!("repl.input_error", error = error));
                break;
            }
        };

        if input.is_empty() {
            continue;
        }

        if is_help_trigger(&input) {
            append_history_or_warn(&mut ctx.command_history, &input, reporter).await;
            if handle_help_menu(&ctx.config, reporter) {
                break;
            }
            continue;
        }

        if is_config_trigger(&input) {
            append_history_or_warn(&mut ctx.command_history, &input, reporter).await;

            if input.eq_ignore_ascii_case("geli-reset-config") {
                match ShellConfig::reset().await {
                    Ok(()) => {
                        reporter.info(&t!("config.reset_ok"));
                    }
                    Err(error) => {
                        reporter.error(&t!("config.reset_failed", error = error));
                    }
                }
            } else if handle_config_menu(&mut ctx.config, reporter).await {
                completion_pool = build_completion_pool(ctx.map.as_ref(), &ctx.config, &ctx.subsystem);
            }
            continue;
        }

        if let Some(special) = parse_special_command(&input) {
            handle_special_command(special, reporter);
            continue;
        }

        if let Some(action) = parse_geli_internal_command(&input) {
            append_history_or_warn(&mut ctx.command_history, &input, reporter).await;
            handle_geli_internal_command(action, &mut ctx.config, reporter).await;
            continue;
        }

        append_history_or_warn(&mut ctx.command_history, &input, reporter).await;

        if process_regular_command(
            &input,
            &ctx.config,
            ctx.guard.as_ref(),
            &ctx.pipeline,
            &ctx.executor,
            &ctx.exec_config,
            &mut ctx.builtins,
            reporter,
            &mut seen_once,
        )
        .await
        {
            drain_crossterm_events(reporter).await;
        }
    }
}
