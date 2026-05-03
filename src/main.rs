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

fn main() {
    // ── Anti-Inception: prevenir ejecución anidada ────────────
    // Checked and set here, before the tokio runtime is built, so no worker
    // threads exist yet — this is the only genuinely safe moment to call set_var.
    if std::env::var("GELISHELL_ACTIVE").is_ok() {
        eprintln!("{}", geli_shell::shell::i18n::t("startup.already_running"));
        std::process::exit(1);
    }
    // SAFETY: no threads exist at this point — the tokio runtime has not yet
    // been built.  This write happens-before any concurrent env read.
    unsafe {
        std::env::set_var("GELISHELL_ACTIVE", "1");
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
        .block_on(async_main());
}

async fn async_main() {
    // ── CLI Flags pareser ───────────────────────────────────
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        handle_cli_args(&args[1..]).await;
        return;
    }

    let bootstrap_reporter = StderrReporter::default();

    bootstrap_runtime_layout(&bootstrap_reporter).await;

    let config = load_or_init_config(&bootstrap_reporter).await;
    let reporter = StderrReporter::new(config.behavior.reporter_level);
    let lang = geli_shell::shell::i18n::detect_language(&config.behavior.language);
    geli_shell::shell::i18n::init_i18n(&lang);
    let command_history = load_history_or_default(&reporter).await;
    let map = init_command_map_or_exit(&reporter).await;
    let subsystem = resolve_subsystem(&config, &reporter);

    let pipeline = TranslationPipeline::new(Arc::clone(&map), subsystem.clone());
    let executor = Executor::new(subsystem.clone());
    let guard = Box::new(default_guard_normalized(Arc::clone(&map)));
    let exec_config = config.to_executor_config();
    let builtins = BuiltinRegistry::new();
    apply_visual_settings(&config, &reporter);

    reporter.info(&geli_shell::t!("startup.ready"));
    use geli_shell::shell::banner::print_banner;
    print_banner("0.1.0", &mut std::io::stdout());
    reporter.info(&geli_shell::t!("startup.subsystem", subsystem = subsystem));

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
