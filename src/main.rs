mod cli;
mod repl;
mod setup;
mod utils;

mod handlers;


use cli::handle_cli_args;
use geli_shell::shell::{
    builtins::BuiltinRegistry,
    executor::Executor,
    guard::default_guard_normalized,
    reporter::{Reporter, StderrReporter},
    translator::TranslationPipeline,
};
use setup::{
    bootstrap_runtime_layout, init_command_map_or_exit, load_history_or_default,
    load_or_init_config, resolve_subsystem,
};
use std::sync::Arc;
use utils::apply_visual_settings;

#[tokio::main]
async fn main() {
    // ── Anti-Inception: prevenir ejecución anidada ────────────
    if std::env::var("GELISHELL_ACTIVE").is_ok() {
        eprintln!("Error: GeliShell ya está en ejecución.");
        std::process::exit(1);
    }
    // SAFETY: called once before the tokio runtime spawns worker threads,
    // so no concurrent reads of the environment can occur.
    unsafe {
        std::env::set_var("GELISHELL_ACTIVE", "1");
    }

    // ── Parseo de flags CLI ───────────────────────────────────
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        handle_cli_args(&args[1..]).await;
        return;
    }

    let reporter = StderrReporter::new();

    bootstrap_runtime_layout(&reporter).await;

    let config = load_or_init_config(&reporter).await;
    let command_history = load_history_or_default(&reporter).await;
    let map = init_command_map_or_exit(&reporter).await;
    let subsystem = resolve_subsystem(&config, &reporter);

    let pipeline = TranslationPipeline::new(Arc::clone(&map), subsystem.clone());
    let executor = Executor::new(subsystem.clone());
    let guard = Box::new(default_guard_normalized(Arc::clone(&map)));
    let exec_config = config.to_executor_config();
    let builtins = BuiltinRegistry::new();
    apply_visual_settings(&config, &reporter);

    reporter.info("GeliShell ready");
    use geli_shell::shell::banner::print_banner;
    print_banner("0.1.0", &mut std::io::stdout());
    reporter.info(&format!("subsystem: {subsystem}"));

    let ctx = repl::ReplContext {
        config,
        command_history,
        map,
        subsystem,
        pipeline,
        executor,
        exec_config,
        guard,
        builtins,
    };
    repl::run_repl(ctx, &reporter).await;
}
