use geli_shell::shell::{
    config::{
        ConfigError, ShellConfig, bootstrap::ensure_runtime_layout,
        first_run::run_first_run_wizard, history_store::PersistentCommandHistory,
    },
    reporter::Reporter,
    translator::{self, CommandMap, Subsystem},
};
use geli_shell::t;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub async fn bootstrap_runtime_layout(reporter: &dyn Reporter) {
    match ensure_runtime_layout().await {
        Ok(report) => {
            if !report.migrated_legacy_files.is_empty() {
                reporter.info(&t!(
                    "bootstrap.migrated",
                    dir = ShellConfig::geli_config_dir().display(),
                    files = report.migrated_legacy_files.join(", ")
                ));
            }
            if !report.seeded_model_files.is_empty() {
                reporter.info(&t!(
                    "bootstrap.seeded",
                    dir = ShellConfig::assistant_models_dir().display(),
                    files = report.seeded_model_files.join(", ")
                ));
            }
        }
        Err(error) => {
            reporter.warn(&t!("bootstrap.failed", error = error));
        }
    }
}

pub async fn load_or_init_config(reporter: &dyn Reporter) -> ShellConfig {
    match ShellConfig::load_async().await {
        Ok(cfg) => cfg,
        Err(ConfigError::NotFound) => {
            let cfg = match run_first_run_wizard() {
                Ok(cfg) => cfg,
                Err(error) => {
                    reporter.warn(&t!("config.wizard_failed", error = error));
                    ShellConfig::default()
                }
            };

            if let Err(error) = cfg.save_async().await {
                reporter.warn(&t!("config.save_failed", error = error));
            }
            cfg
        }
        Err(ConfigError::Parse(error)) => {
            reporter.error(&t!("config.parse_error", error = error));
            ShellConfig::default()
        }
        Err(error) => {
            reporter.warn(&t!("config.load_error", error = error));
            ShellConfig::default()
        }
    }
}

pub async fn load_history_or_default(reporter: &dyn Reporter) -> PersistentCommandHistory {
    match PersistentCommandHistory::load_async().await {
        Ok(history) => history,
        Err(error) => {
            reporter.warn(&t!("history.load_failed", error = error));
            PersistentCommandHistory::default()
        }
    }
}

pub async fn init_command_map_or_exit(reporter: &dyn Reporter) -> Arc<CommandMap> {
    let (result, command_map_source) = match load_command_map_for_startup().await {
        Ok(loaded) => loaded,
        Err(error) => {
            reporter.error(&error);
            std::process::exit(1);
        }
    };

    reporter.info(&t!(
        "commands.loaded",
        count = result.map.len(),
        source = command_map_source
    ));
    result.report(reporter);

    Arc::new(result.map)
}

pub fn resolve_subsystem(config: &ShellConfig, reporter: &dyn Reporter) -> Subsystem {
    if config.has_subsystem_override() {
        Subsystem::from_str(&config.subsystem.override_subsystem)
            .unwrap_or_else(|| Subsystem::detect(reporter))
    } else {
        Subsystem::detect(reporter)
    }
}

async fn load_command_map_for_startup() -> Result<(translator::LoadResult, String), String> {
    for path in command_map_runtime_candidates() {
        let Ok(metadata) = tokio::fs::metadata(&path).await else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }

        let raw = tokio::fs::read_to_string(&path)
            .await
            .map_err(|error| t!("commands.read_failed", path = path.display(), error = error))?;
        let parsed = translator::load_from_str(&raw).map_err(|error| {
            t!(
                "commands.parse_failed",
                path = path.display(),
                error = error
            )
        })?;

        return Ok((parsed, format!("runtime ({})", path.display())));
    }

    let embedded = translator::load().map_err(|error| error.to_string())?;
    Ok((embedded, "embedded".to_owned()))
}

fn command_map_runtime_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(raw_path) = std::env::var("GELI_COMMANDS_PATH") {
        if !raw_path.trim().is_empty() {
            candidates.push(PathBuf::from(raw_path.trim()));
        }
    }

    candidates.push(ShellConfig::geli_config_dir().join("commands.toml"));

    if let Ok(cwd) = std::env::current_dir() {
        append_command_map_patterns(&cwd, &mut candidates);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            append_command_map_patterns(exe_dir, &mut candidates);
            if let Some(parent) = exe_dir.parent() {
                append_command_map_patterns(parent, &mut candidates);
            }
        }
    }

    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for candidate in candidates {
        let key = candidate
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();
        if seen.insert(key) {
            deduped.push(candidate);
        }
    }
    deduped
}

fn append_command_map_patterns(base: &Path, out: &mut Vec<PathBuf>) {
    out.push(base.join("commands.toml"));
    out.push(base.join("commands").join("commands.toml"));
    out.push(base.join("src").join("commands").join("commands.toml"));
}
