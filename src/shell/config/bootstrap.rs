use super::ShellConfig;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default)]
pub struct RuntimeBootstrapReport {
    pub migrated_legacy_files: Vec<String>,
    pub seeded_model_files: Vec<String>,
}

pub async fn ensure_runtime_layout() -> Result<RuntimeBootstrapReport, std::io::Error> {
    let config_dir = ShellConfig::geli_config_dir();
    let docs_dir = ShellConfig::assistant_docs_dir();
    let models_dir = ShellConfig::assistant_models_dir();

    tokio::fs::create_dir_all(&config_dir).await?;
    tokio::fs::create_dir_all(&docs_dir).await?;
    tokio::fs::create_dir_all(&models_dir).await?;

    let mut report = RuntimeBootstrapReport::default();
    report.migrated_legacy_files = migrate_legacy_files(&config_dir).await?;

    let roots = candidate_roots();

    let docs_db_target = ShellConfig::assistant_docs_db_path();
    let docs_db_candidates = prepend_env_sources(
        &["GELI_DOCS_DB_SOURCE", "GELI_DOCS_DB_PATH"],
        file_candidates(&roots, &["docs.db"]),
    );
    if let Some(seed) = seed_from_candidates(&docs_db_target, &docs_db_candidates).await? {
        report
            .seeded_model_files
            .push(format!("docs.db <= {}", seed.display()));
    }

    let sqlite_vec_target = models_dir.join(default_sqlite_vec_filename());
    let sqlite_vec_candidates = prepend_env_sources(
        &["GELI_SQLITE_VEC_SOURCE", "GELI_SQLITE_VEC_PATH"],
        file_candidates(&roots, sqlite_vec_filenames()),
    );
    if let Some(seed) = seed_from_candidates(&sqlite_vec_target, &sqlite_vec_candidates).await? {
        report.seeded_model_files.push(format!(
            "{} <= {}",
            sqlite_vec_target.display(),
            seed.display()
        ));
    }

    let dbjson_target = models_dir.join("dbjson");
    let dbjson_candidates = prepend_env_sources(
        &["GELI_DBJSON_SOURCE"],
        file_candidates(&roots, &["dbjson", "db.json", "dbjson.json"]),
    );
    if let Some(seed) = seed_from_candidates(&dbjson_target, &dbjson_candidates).await? {
        report
            .seeded_model_files
            .push(format!("dbjson <= {}", seed.display()));
    }

    Ok(report)
}

fn legacy_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("geliShell")
}

async fn migrate_legacy_files(target_config_dir: &Path) -> Result<Vec<String>, std::io::Error> {
    let legacy = legacy_config_dir();
    if legacy == target_config_dir {
        return Ok(Vec::new());
    }
    if tokio::fs::metadata(&legacy).await.is_err() {
        return Ok(Vec::new());
    }

    let docs_db_relative = ShellConfig::docs_db_path(Path::new(""))
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_owned();
    let relative_files = [
        "config.toml".to_owned(),
        "history.txt".to_owned(),
        docs_db_relative,
        "models/vec0.dll".to_owned(),
        "models/dbjson".to_owned(),
        "models/db.json".to_owned(),
    ];

    let mut migrated = Vec::new();
    for relative in &relative_files {
        let source = join_relative(&legacy, relative);
        let target = join_relative(target_config_dir, relative);
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        if tokio::fs::metadata(&source).await.is_err() || tokio::fs::metadata(&target).await.is_ok()
        {
            continue;
        }
        tokio::fs::copy(&source, &target).await?;
        migrated.push(relative.replace('\\', "/"));
    }

    Ok(migrated)
}

fn join_relative(base: &Path, relative: &str) -> PathBuf {
    relative
        .split(['\\', '/'])
        .fold(base.to_path_buf(), |acc, segment| acc.join(segment))
}

fn prepend_env_sources(var_names: &[&str], mut candidates: Vec<PathBuf>) -> Vec<PathBuf> {
    for var_name in var_names.iter().rev() {
        if let Ok(value) = std::env::var(var_name) {
            if !value.trim().is_empty() {
                candidates.insert(0, PathBuf::from(value));
            }
        }
    }
    candidates
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut ordered = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        ordered.push(cwd);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            ordered.push(exe_dir.to_path_buf());
            if let Some(parent) = exe_dir.parent() {
                ordered.push(parent.to_path_buf());
                if let Some(grand_parent) = parent.parent() {
                    ordered.push(grand_parent.to_path_buf());
                }
            }
        }
    }

    if let Ok(install_root) = std::env::var("GELI_INSTALL_ROOT") {
        if !install_root.trim().is_empty() {
            ordered.push(PathBuf::from(install_root));
        }
    }

    let mut unique = Vec::new();
    let mut seen = BTreeSet::new();
    for root in ordered {
        let key = root.to_string_lossy().to_ascii_lowercase();
        if seen.insert(key) {
            unique.push(root);
        }
    }
    unique
}

fn file_candidates(roots: &[PathBuf], names: &[&str]) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();

    for root in roots {
        for name in names {
            for candidate in [
                root.join(name),
                root.join("models").join(name),
                root.join("assets").join(name),
                root.join(".config")
                    .join("geliShell")
                    .join("models")
                    .join(name),
            ] {
                let key = candidate.to_string_lossy().to_ascii_lowercase();
                if seen.insert(key) {
                    candidates.push(candidate);
                }
            }
        }
    }
    candidates
}

fn sqlite_vec_filenames() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["vec0.dll", "sqlite_vec.dll", "sqlite-vec.dll"]
    }

    #[cfg(target_os = "linux")]
    {
        &["vec0.so", "sqlite_vec.so", "sqlite-vec.so", "vec0"]
    }

    #[cfg(target_os = "macos")]
    {
        &["vec0.dylib", "sqlite_vec.dylib", "sqlite-vec.dylib", "vec0"]
    }
}

fn default_sqlite_vec_filename() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "vec0.dll"
    }

    #[cfg(target_os = "linux")]
    {
        "vec0.so"
    }

    #[cfg(target_os = "macos")]
    {
        "vec0.dylib"
    }
}

async fn seed_from_candidates(
    target: &Path,
    candidates: &[PathBuf],
) -> Result<Option<PathBuf>, std::io::Error> {
    if tokio::fs::metadata(target).await.is_ok() {
        return Ok(None);
    }

    for source in candidates {
        if source == target {
            continue;
        }

        let Ok(metadata) = tokio::fs::metadata(source).await else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }

        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(source, target).await?;
        return Ok(Some(source.clone()));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn seed_from_candidates_copies_when_target_missing() {
        let base = unique_test_dir("bootstrap_seed");
        tokio::fs::create_dir_all(&base)
            .await
            .expect("test base dir should be created");
        let source = base.join("source.docs.db");
        let target = ShellConfig::docs_db_path(&base);
        tokio::fs::write(&source, b"test-db")
            .await
            .expect("source file should be written");

        let copied = seed_from_candidates(&target, &[source.clone()])
            .await
            .expect("seed copy should succeed");
        assert_eq!(copied, Some(source));
        let copied_bytes = tokio::fs::read(&target)
            .await
            .expect("target should exist after seed");
        assert_eq!(copied_bytes, b"test-db");

        let _ = tokio::fs::remove_dir_all(&base).await;
    }

    #[tokio::test]
    async fn seed_from_candidates_does_not_override_existing_target() {
        let base = unique_test_dir("bootstrap_no_override");
        tokio::fs::create_dir_all(&base)
            .await
            .expect("test base dir should be created");
        let source = base.join("source.docs.db");
        let target = ShellConfig::docs_db_path(&base);
        tokio::fs::create_dir_all(
            target
                .parent()
                .expect("target parent should exist for create_dir_all"),
        )
        .await
        .expect("docs dir should be created");
        tokio::fs::write(&source, b"new")
            .await
            .expect("source file should be written");
        tokio::fs::write(&target, b"existing")
            .await
            .expect("target file should be written");

        let copied = seed_from_candidates(&target, &[source])
            .await
            .expect("seed should not fail when target exists");
        assert!(copied.is_none());
        let bytes = tokio::fs::read(&target)
            .await
            .expect("target should still exist");
        assert_eq!(bytes, b"existing");

        let _ = tokio::fs::remove_dir_all(&base).await;
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_millis();
        std::env::temp_dir().join(format!("geli_shell_{prefix}_{millis}"))
    }
}
