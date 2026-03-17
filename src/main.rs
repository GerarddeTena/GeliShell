use crossterm::{
    execute,
    style::{Color, SetBackgroundColor, SetForegroundColor},
};
use geli_shell::{
    Guard, Reporter, StderrReporter,
    parser::{lexer::Lexer, parser::Parser},
    shell::{
        assistant::{AssistantRuntime, suggest::AssistantSuggestion},
        builtins::{BuiltinRegistry, BuiltinResult, clear::clear_console_buffer},
        config::first_run::run_first_run_wizard,
        config::{
            ConfigError, SelectorMode, ShellConfig, VisualConfig, bootstrap::ensure_runtime_layout,
            history_store::PersistentCommandHistory,
        },
        executor::Executor,
        guard::default_guard,
        reporter::SilentReporter,
        translator::{self, CommandMap, Subsystem, TranslationPipeline},
        tui::{
            assistant_menu::{
                AssistantMenuSelection, show_assistant_error_panel, show_assistant_menu,
                show_model_bootstrap_progress,
            },
            config_menu::{ConfigMenuSelection, show_config_menu},
            help_menu::{HelpMenuAction, show_help_menu},
            repl_input::{ReplInputAction, SpecialCommand, parse_special_command, read_repl_input},
        },
    },
};
use std::collections::BTreeSet;
use std::io::Write;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::signal;

#[tokio::main]
async fn main() {
    let reporter = StderrReporter::new();

    match ensure_runtime_layout().await {
        Ok(report) => {
            if !report.migrated_legacy_files.is_empty() {
                reporter.info(&format!(
                    "migrated legacy config files to {}: {}",
                    ShellConfig::geli_config_dir().display(),
                    report.migrated_legacy_files.join(", ")
                ));
            }
            if !report.seeded_model_files.is_empty() {
                reporter.info(&format!(
                    "initialized assistant model assets in {}: {}",
                    ShellConfig::assistant_models_dir().display(),
                    report.seeded_model_files.join(", ")
                ));
            }
        }
        Err(error) => {
            reporter.warn(&format!("bootstrap layout failed: {error}"));
        }
    }

    // ── Carga o crea la configuración ────────────────────────
    let mut config = match ShellConfig::load_async().await {
        Ok(cfg) => cfg,
        Err(ConfigError::NotFound) => {
            let cfg = match run_first_run_wizard() {
                Ok(cfg) => cfg,
                Err(e) => {
                    reporter.warn(&format!("wizard failed: {e} — using defaults"));
                    ShellConfig::default()
                }
            };

            if let Err(e) = cfg.save_async().await {
                reporter.warn(&format!("could not save config: {e}"));
            }
            cfg
        }
        Err(ConfigError::Parse(e)) => {
            reporter.error(&format!(
                "\x1b[31mconfig parse error: {e} — using fallback defaults\x1b[0m"
            ));
            ShellConfig::default()
        }
        Err(e) => {
            reporter.warn(&format!("config error: {e} — using defaults"));
            ShellConfig::default()
        }
    };

    // ── Carga historial persistente ───────────────────────────
    let mut command_history = match PersistentCommandHistory::load_async().await {
        Ok(history) => history,
        Err(e) => {
            reporter.warn(&format!(
                "history load failed: {e} — continuing with empty history"
            ));
            PersistentCommandHistory::default()
        }
    };

    // ── Carga el mapa de comandos ─────────────────────────────
    let result = match translator::load() {
        Ok(r) => r,
        Err(e) => {
            reporter.error(&e.to_string());
            std::process::exit(1);
        }
    };
    result.report(&reporter);

    // ── Inicializa el sistema ─────────────────────────────────
    let map = Arc::new(result.map);

    // Subsistema: override en config > auto-detect
    let subsystem = if config.has_subsystem_override() {
        Subsystem::from_str(&config.subsystem.override_subsystem)
            .unwrap_or_else(|| Subsystem::detect(&reporter))
    } else {
        Subsystem::detect(&reporter)
    };

    let pipeline = TranslationPipeline::new(Arc::clone(&map), subsystem.clone());
    let executor = Executor::new(subsystem.clone());
    let guard = default_guard();
    let exec_config = config.to_executor_config();
    let mut builtins = BuiltinRegistry::new();
    let mut completion_pool = build_completion_pool(map.as_ref(), &config);
    let mut assistant = AssistantRuntime::new(&config);

    apply_visual_settings(&config, &reporter);

    reporter.info("GeliShell ready");
    use geli_shell::shell::banner::print_banner;
    print_banner("0.1.0");
    reporter.info(&format!("subsystem: {subsystem}"));

    // ── REPL ──────────────────────────────────────────────────
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
                if handle_help_menu(&config, &reporter) {
                    break;
                }
                continue;
            }
            Ok(ReplInputAction::OpenConfig) => {
                if handle_config_menu(&mut config, &reporter).await {
                    completion_pool = build_completion_pool(map.as_ref(), &config);
                    assistant.refresh_config(&config);
                }
                continue;
            }
            Ok(ReplInputAction::OpenAssistant) => {
                handle_assistant(&mut assistant, &config, &reporter).await;
                continue;
            }
            Ok(ReplInputAction::Clear) => {
                run_clear(&config, &reporter);
                continue;
            }
            Ok(ReplInputAction::Search) => {
                handle_special_command(SpecialCommand::Search, &reporter);
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
            if let Err(error) = command_history.append_async(&input).await {
                reporter.warn(&format!("history append failed: {error}"));
            }
            if handle_help_menu(&config, &reporter) {
                break;
            }
            continue;
        }

        if is_config_trigger(&input) {
            if let Err(error) = command_history.append_async(&input).await {
                reporter.warn(&format!("history append failed: {error}"));
            }
            if handle_config_menu(&mut config, &reporter).await {
                completion_pool = build_completion_pool(map.as_ref(), &config);
                assistant.refresh_config(&config);
            }
            continue;
        }

        if is_assistant_trigger(&input) {
            if let Err(error) = command_history.append_async(&input).await {
                reporter.warn(&format!("history append failed: {error}"));
            }
            handle_assistant(&mut assistant, &config, &reporter).await;
            continue;
        }

        if let Some(special) = parse_special_command(&input) {
            handle_special_command(special, &reporter);
            continue;
        }

        if let Err(error) = command_history.append_async(&input).await {
            reporter.warn(&format!("history append failed: {error}"));
        }

        builtins.push_history(input.clone());

        let expanded_input = expand_custom_command(&input, &config);

        // ── Lexer → Parser ────────────────────────────────────
        let tokens = match Lexer::new(&expanded_input).tokenize() {
            Ok(t) => t,
            Err(e) => {
                reporter.error(&e.to_string());
                continue;
            }
        };
        let ast = match Parser::new(tokens).parse() {
            Ok(a) => a,
            Err(e) => {
                reporter.error(&e.to_string());
                continue;
            }
        };

        // ── Builtins ──────────────────────────────────────────
        match builtins.try_execute(&ast, &reporter) {
            BuiltinResult::Handled => {
                builtins.record_g_visit();
                if expanded_input.trim() == "clear" {
                    apply_visual_settings(&config, &reporter);
                }
                continue;
            }
            BuiltinResult::Exit(code) => std::process::exit(code),
            BuiltinResult::NotABuiltin => {}
        }

        // ── Guard ─────────────────────────────────────────────
        if let Err(e) = guard.check(&ast) {
            reporter.error(&e.to_string());
            continue;
        }

        // ── Pipeline → ResolvedCommand ────────────────────────
        let command = match pipeline.run(&ast, &SilentReporter::new()) {
            Ok(c) => c,
            Err(e) => {
                reporter.error(&e.to_string());
                continue;
            }
        };

        let command_str = command.clone();
        let _final_command = match config.behavior.selector_mode {
            SelectorMode::Auto => command,
            SelectorMode::Always | SelectorMode::Once => command,
        };

        let interactive_tty = Executor::requires_tty(&command_str);

        // ── Executor con Ctrl+C ───────────────────────────────
        if interactive_tty {
            tokio::select! {
                result = executor.run(&command_str, &exec_config, &reporter) => {
                    match result {
                        Ok(res) => {
                            builtins.record_g_visit();

                            if !res.success() {
                                reporter.warn(&format!(
                                    "exit code: {}", res.exit_code
                                ));
                            }
                        }
                        Err(e) => reporter.error(&e.to_string()),
                    }
                }
                _ = signal::ctrl_c() => {
                    println!();
                    reporter.warn("^C — command cancelled");
                }
            }
        } else {
            tokio::select! {
                result = executor.run(&command_str, &exec_config, &reporter) => {
                    match result {
                        Ok(res) => {
                            builtins.record_g_visit();

                            if !res.success() {
                                reporter.warn(&format!(
                                    "exit code: {}", res.exit_code
                                ));
                            }
                        }
                        Err(e) => reporter.error(&e.to_string()),
                    }
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
                    handle_special_command(special, &reporter);
                }
            }
        }
    }
}

async fn handle_config_menu(config: &mut ShellConfig, reporter: &dyn Reporter) -> bool {
    match show_config_menu(&config.visual) {
        Ok(ConfigMenuSelection::Closed) => false,
        Ok(ConfigMenuSelection::UpdatedVisual(visual)) => {
            config.visual = visual;
            apply_visual_settings(config, reporter);

            if let Err(error) = config.save_async().await {
                reporter.error(&format!("config save failed: {error}"));
            } else {
                reporter.info("config updated and persisted");
            }
            true
        }
        Ok(ConfigMenuSelection::TomlEditor) => {
            reporter.warn("WARNING: editing command toml can break the shell if invalid");
            let commands_path = std::env::current_dir()
                .unwrap_or_default()
                .join("src")
                .join("commands")
                .join("commands.toml");
            reporter.info(&format!("customization file: {}", commands_path.display()));
            false
        }
        Err(error) => {
            reporter.error(&format!("config menu failed: {error}"));
            false
        }
    }
}

fn build_completion_pool(map: &CommandMap, config: &ShellConfig) -> Vec<String> {
    let mut set = BTreeSet::new();

    for builtin in [
        "cd",
        "clear",
        "exit",
        "export",
        "history",
        "source",
        "unset",
        "g",
        "gerisabet",
        "geli-helpme",
        "geli-config-me",
        "geli-",
    ] {
        set.insert(builtin.to_owned());
    }

    for cmd in map.iter() {
        set.insert(cmd.name.clone());

        if let Some(entry) = &cmd.translate.bash {
            if !entry.exact.trim().is_empty() {
                set.insert(entry.exact.clone());
            }
            for suggestion in &entry.suggestions {
                if !suggestion.trim().is_empty() {
                    set.insert(suggestion.clone());
                }
            }
        }
        if let Some(entry) = &cmd.translate.powershell {
            if !entry.exact.trim().is_empty() {
                set.insert(entry.exact.clone());
            }
            for suggestion in &entry.suggestions {
                if !suggestion.trim().is_empty() {
                    set.insert(suggestion.clone());
                }
            }
        }
        if let Some(entry) = &cmd.translate.cmd {
            if !entry.exact.trim().is_empty() {
                set.insert(entry.exact.clone());
            }
            for suggestion in &entry.suggestions {
                if !suggestion.trim().is_empty() {
                    set.insert(suggestion.clone());
                }
            }
        }
    }

    for custom in &config.customization.custom_commands {
        if !custom.name.trim().is_empty() {
            set.insert(custom.name.trim().to_owned());
        }
    }

    set.into_iter().collect()
}

fn expand_custom_command(input: &str, config: &ShellConfig) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let Some(head) = parts.next() else {
        return trimmed.to_owned();
    };

    let Some(custom) = config
        .customization
        .custom_commands
        .iter()
        .find(|entry| entry.name.trim() == head && !entry.template.trim().is_empty())
    else {
        return trimmed.to_owned();
    };

    let tail = parts.next().unwrap_or("").trim();
    if tail.is_empty() {
        custom.template.trim().to_owned()
    } else {
        format!("{} {}", custom.template.trim(), tail)
    }
}

fn apply_visual_settings(config: &ShellConfig, reporter: &dyn Reporter) {
    let mut out = std::io::stdout();
    if let Err(error) = execute!(
        out,
        SetForegroundColor(Color::AnsiValue(config.visual.terminal_foreground_ansi256)),
        SetBackgroundColor(Color::AnsiValue(config.visual.terminal_background_ansi256)),
    ) {
        reporter.warn(&format!("visual apply failed: {error}"));
    }

    if let Err(error) = write!(out, "\x1b]50;{}\x07", config.visual.font_family) {
        reporter.warn(&format!("font apply failed: {error}"));
    }
    if let Err(error) = out.flush() {
        reporter.warn(&format!("visual flush failed: {error}"));
    }
}

async fn wait_for_runtime_special_command() -> SpecialCommand {
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            Ok(_) => {
                if let Some(command) = parse_special_command(line.trim()) {
                    return command;
                }
            }
        }
    }
}

fn run_clear(config: &ShellConfig, reporter: &dyn Reporter) {
    if let Err(error) = clear_console_buffer() {
        reporter.error(&format!("clear failed: {error}"));
        return;
    }
    apply_visual_settings(config, reporter);
}

fn is_help_trigger(input: &str) -> bool {
    matches!(input, "geli-helpme" | "^?" | "^H" | "\u{8}" | "\u{7f}")
}

fn is_config_trigger(input: &str) -> bool {
    input.eq_ignore_ascii_case("geli-config-me")
}

fn is_assistant_trigger(input: &str) -> bool {
    input.eq_ignore_ascii_case("gerisabet")
}

fn handle_help_menu(config: &ShellConfig, reporter: &dyn Reporter) -> bool {
    match show_help_menu() {
        Ok(HelpMenuAction::None) => false,
        Ok(HelpMenuAction::Clear) => {
            run_clear(config, reporter);
            false
        }
        Ok(HelpMenuAction::Exit) => {
            reporter.info("goodbye");
            true
        }
        Ok(HelpMenuAction::Stop) => {
            handle_special_command(SpecialCommand::Stop, reporter);
            false
        }
        Ok(HelpMenuAction::Search) => {
            handle_special_command(SpecialCommand::Search, reporter);
            false
        }
        Err(error) => {
            reporter.error(&format!("help menu failed: {error}"));
            false
        }
    }
}

fn handle_special_command(command: SpecialCommand, reporter: &dyn Reporter) {
    match command {
        SpecialCommand::Stop => {
            reporter.warn(":stop* intercepted — use Ctrl+C to interrupt running command");
        }
        SpecialCommand::Search => {
            reporter.info(":search* intercepted — interactive search UI is active as skeleton");
        }
    }
}

async fn handle_assistant(
    assistant: &mut AssistantRuntime,
    config: &ShellConfig,
    reporter: &dyn Reporter,
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
        return;
    }

    let selection = match show_assistant_menu() {
        Ok(selection) => selection,
        Err(error) => {
            reporter.error(&format!("assistant panel failed: {error}"));
            assistant.release_resources().await;
            return;
        }
    };

    let AssistantMenuSelection::Selected { parameter, filter } = selection else {
        assistant.release_resources().await;
        return;
    };

    match assistant.run_parameter(parameter, &filter).await {
        Ok(suggestion) => report_assistant_suggestion(suggestion, reporter),
        Err(error) => {
            if let Err(ui_error) = show_assistant_error_panel(&error.to_string()) {
                reporter.error(&format!("assistant error panel failed: {ui_error}"));
            }
            reporter.error(&format!("assistant inference failed: {error}"));
        }
    }
    assistant.release_resources().await;
}

fn report_assistant_suggestion(suggestion: AssistantSuggestion, reporter: &dyn Reporter) {
    reporter.info(&suggestion.title);
    for line in suggestion.body.lines() {
        reporter.info(line);
    }

    if !suggestion.sources.is_empty() {
        reporter.info("assistant rag sources:");
        for source in suggestion.sources {
            reporter.info(&format!("  - {source}"));
        }
    }
}

fn render_prompt(subsystem: &Subsystem, visual: &VisualConfig) -> String {
    let cwd = match std::env::current_dir() {
        Err(_) => "?".to_owned(),
        Ok(path) => {
            let normalized = path.to_string_lossy().replace('\\', "/");

            let home_var = if cfg!(target_os = "windows") {
                std::env::var("USERPROFILE")
            } else {
                std::env::var("HOME")
            };

            match home_var {
                Ok(home) => {
                    let home_normalized = home.replace('\\', "/");
                    if normalized.starts_with(&home_normalized) {
                        let stripped = &normalized[home_normalized.len()..];
                        if stripped.is_empty() {
                            "~".to_owned()
                        } else {
                            format!("~{stripped}")
                        }
                    } else {
                        normalized
                    }
                }
                Err(_) => normalized,
            }
        }
    };

    let path = format!("\x1b[38;5;{}m", visual.prompt_path_ansi256);
    let subsystem_color = format!("\x1b[38;5;{}m", visual.prompt_subsystem_ansi256);
    let name = format!("\x1b[38;5;{}m", visual.prompt_name_ansi256);
    let dim = format!("\x1b[38;5;{}m", visual.prompt_dim_ansi256);
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";

    format!(
        "{path}{bold}{cwd}{reset} \
         {dim}_{reset}{subsystem_color}{}{reset}{dim}_{reset} \
         {name}{bold}Geli$hell{reset}{dim}>{reset} ",
        subsystem.as_str().to_uppercase(),
    )
}
