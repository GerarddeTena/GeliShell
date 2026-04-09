use super::ShellConfig;
use crate::shell::reporter::Reporter;
use crate::t;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

// ── sqlite-vec release configuration ────────────────────────────
const SQLITE_VEC_RELEASE_API: &str =
    "https://api.github.com/repos/asg017/sqlite-vec/releases/latest";

// ── GeliShell release configuration ─────────────────────────────
const GELISHELL_RELEASE_API: &str =
    "https://api.github.com/repos/GerarddeTena/GeliShell/releases/latest";

#[derive(Debug, Clone, Default)]
pub struct RuntimeBootstrapReport {
    pub migrated_legacy_files: Vec<String>,
    pub seeded_model_files: Vec<String>,
}

pub async fn ensure_runtime_layout(
    reporter: &dyn Reporter,
) -> Result<RuntimeBootstrapReport, std::io::Error> {
    let config_dir = ShellConfig::geli_config_dir();
    let docs_dir = ShellConfig::assistant_docs_dir();
    let models_dir = ShellConfig::assistant_models_dir();

    tokio::fs::create_dir_all(&config_dir).await?;
    tokio::fs::create_dir_all(&docs_dir).await?;
    tokio::fs::create_dir_all(&models_dir).await?;

    let mut report = RuntimeBootstrapReport {
        migrated_legacy_files: migrate_legacy_files(&config_dir).await?,
        ..Default::default()
    };

    let roots = candidate_roots();

    // ── docs.db: seed from local candidates, then try HTTP download ──
    let docs_db_target = ShellConfig::assistant_docs_db_path();
    let docs_db_candidates = prepend_env_sources(
        &["GELI_DOCS_DB_SOURCE", "GELI_DOCS_DB_PATH"],
        file_candidates(&roots, &["docs.db"]),
    );
    if let Some(seed) = seed_from_candidates(&docs_db_target, &docs_db_candidates).await? {
        report
            .seeded_model_files
            .push(format!("docs.db <= {}", seed.display()));
    } else if !file_exists(&docs_db_target).await {
        match download_docs_db_if_absent(&docs_db_target, reporter).await {
            Ok(true) => {
                report
                    .seeded_model_files
                    .push("docs.db <= GitHub release".to_owned());
            }
            Ok(false) => {}
            Err(error) => {
                reporter.warn(&t!(
                    "bootstrap.download_failed",
                    name = "docs.db",
                    error = error
                ));
            }
        }
    }

    // ── sqlite-vec: seed from local candidates, then try HTTP download ──
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
    } else if !file_exists(&sqlite_vec_target).await {
        match download_sqlite_vec_if_absent(&sqlite_vec_target, reporter).await {
            Ok(true) => {
                report
                    .seeded_model_files
                    .push(format!("{} <= GitHub release", sqlite_vec_target.display()));
            }
            Ok(false) => {}
            Err(error) => {
                reporter.warn(&t!(
                    "bootstrap.download_failed",
                    name = default_sqlite_vec_filename(),
                    error = error
                ));
            }
        }
    }

    // ── dbjson: seed from local candidates only (no HTTP fallback) ──
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

// ═══════════════════════════════════════════════════════════════
// HTTP download helpers — non-fatal (warn on failure)
// ═══════════════════════════════════════════════════════════════

/// Downloads sqlite-vec from the latest GitHub release for the current platform.
/// Returns Ok(true) on success, Ok(false) if skipped, Err on failure.
async fn download_sqlite_vec_if_absent(
    target: &Path,
    reporter: &dyn Reporter,
) -> Result<bool, DownloadError> {
    let asset_pattern = sqlite_vec_asset_pattern();
    reporter.info(&t!(
        "bootstrap.downloading_sqlite_vec",
        pattern = asset_pattern
    ));

    let http = build_http_client()?;
    let release: GithubRelease = fetch_github_release(&http, SQLITE_VEC_RELEASE_API).await?;
    let tag = &release.tag_name;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name.contains(asset_pattern) && a.name.ends_with(".tar.gz"))
        .ok_or_else(|| DownloadError::AssetNotFound {
            pattern: asset_pattern.to_owned(),
            tag: tag.clone(),
        })?;

    // Fetch checksums.txt for SHA-256 verification
    let checksums_url =
        format!("https://github.com/asg017/sqlite-vec/releases/download/{tag}/checksums.txt");
    let hash_lookup = lookup_checksum(&http, &checksums_url, &asset.name).await;

    let archive_bytes = download_asset(&http, &asset.browser_download_url).await?;

    match &hash_lookup {
        HashLookup::Found(hash) => {
            verify_sha256(&archive_bytes, hash)?;
            reporter.info(&t!("bootstrap.sha256_verified"));
        }
        HashLookup::Unlisted => {
            reporter.warn(&t!(
                "bootstrap.sha256_not_listed",
                name = default_sqlite_vec_filename()
            ));
        }
        HashLookup::Absent => {}
    }

    let extracted = extract_file_from_tar_gz(&archive_bytes, default_sqlite_vec_filename())?;
    write_with_parent_dirs(target, &extracted).await?;
    reporter.info(&t!(
        "bootstrap.installed",
        name = default_sqlite_vec_filename(),
        tag = tag
    ));
    Ok(true)
}

/// Downloads docs.db from the latest GeliShell GitHub release.
/// Returns Ok(true) on success, Ok(false) if skipped, Err on failure.
async fn download_docs_db_if_absent(
    target: &Path,
    reporter: &dyn Reporter,
) -> Result<bool, DownloadError> {
    reporter.info(&t!("bootstrap.downloading_docs_db"));

    let http = build_http_client()?;
    let release: GithubRelease = fetch_github_release(&http, GELISHELL_RELEASE_API).await?;
    let tag = &release.tag_name;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == "docs.db")
        .ok_or_else(|| DownloadError::AssetNotFound {
            pattern: "docs.db".to_owned(),
            tag: tag.clone(),
        })?;

    // Fetch checksums.txt for SHA-256 verification
    let checksums_url =
        format!("https://github.com/GerarddeTena/GeliShell/releases/download/{tag}/checksums.txt");
    let hash_lookup = lookup_checksum(&http, &checksums_url, &asset.name).await;

    let file_bytes = download_asset(&http, &asset.browser_download_url).await?;

    match &hash_lookup {
        HashLookup::Found(hash) => {
            verify_sha256(&file_bytes, hash)?;
            reporter.info(&t!("bootstrap.sha256_verified"));
        }
        HashLookup::Unlisted => {
            reporter.warn(&t!("bootstrap.sha256_not_listed", name = "docs.db"));
        }
        HashLookup::Absent => {}
    }

    write_with_parent_dirs(target, &file_bytes).await?;
    reporter.info(&t!("bootstrap.installed", name = "docs.db", tag = tag));
    Ok(true)
}

// ═══════════════════════════════════════════════════════════════
// Download infrastructure
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, thiserror::Error)]
enum DownloadError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("GitHub API returned non-success status {status} for {url}")]
    HttpStatus { url: String, status: u16 },

    #[error("no asset matching '{pattern}' found in release {tag}")]
    AssetNotFound { pattern: String, tag: String },

    #[error("SHA-256 mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("tar.gz extraction failed: {0}")]
    Extraction(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, serde::Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

fn build_http_client() -> Result<reqwest::Client, DownloadError> {
    reqwest::Client::builder()
        .user_agent("GeliShell-Bootstrap/0.1")
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(DownloadError::Http)
}

async fn fetch_github_release(
    http: &reqwest::Client,
    api_url: &str,
) -> Result<GithubRelease, DownloadError> {
    let response = http
        .get(api_url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(DownloadError::HttpStatus {
            url: api_url.to_owned(),
            status: response.status().as_u16(),
        });
    }

    Ok(response.json().await?)
}

async fn download_asset(http: &reqwest::Client, url: &str) -> Result<Vec<u8>, DownloadError> {
    let response = http.get(url).send().await?;
    if !response.status().is_success() {
        return Err(DownloadError::HttpStatus {
            url: url.to_owned(),
            status: response.status().as_u16(),
        });
    }
    Ok(response.bytes().await?.to_vec())
}

/// Result of looking up an asset hash in a `checksums.txt` file.
///
/// - `Found(hash)` — hash present; SHA-256 verification is mandatory.
/// - `Absent`      — `checksums.txt` unreachable (404 / network error); skip silently
///   for backward compatibility with releases that predate checksums.
/// - `Unlisted`    — `checksums.txt` was fetched successfully but the asset is not
///   listed; caller should warn the user and proceed without verification.
enum HashLookup {
    Found(String),
    Absent,
    Unlisted,
}

/// Fetches `checksums.txt` and looks up the SHA-256 hash for `asset_name`.
///
/// Matches the GNU `sha256sum` format: `<hash>  <filename>` (two spaces) or
/// `<hash> *<filename>` (binary mode). Matching is exact on the filename token.
async fn lookup_checksum(
    http: &reqwest::Client,
    checksums_url: &str,
    asset_name: &str,
) -> HashLookup {
    let response = match http.get(checksums_url).send().await {
        Ok(r) => r,
        Err(_) => return HashLookup::Absent,
    };

    // 404 = release predates checksums.txt; anything else non-2xx = infra issue.
    // Both cases are treated as Absent to avoid blocking the bootstrap.
    if !response.status().is_success() {
        return HashLookup::Absent;
    }

    let text = match response.text().await {
        Ok(t) => t,
        Err(_) => return HashLookup::Absent,
    };

    for line in text.lines() {
        // Text mode: "<hash>  <filename>"
        if let Some((hash_part, name_part)) = line.split_once("  ")
            && name_part.trim() == asset_name
        {
            return HashLookup::Found(hash_part.trim().to_owned());
        }
        // Binary mode: "<hash> *<filename>"
        if let Some((hash_part, name_part)) = line.split_once(" *")
            && name_part.trim() == asset_name
        {
            return HashLookup::Found(hash_part.trim().to_owned());
        }
    }

    HashLookup::Unlisted
}

fn verify_sha256(data: &[u8], expected_hex: &str) -> Result<(), DownloadError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual_hex = format!("{:x}", hasher.finalize());

    if actual_hex.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(DownloadError::ChecksumMismatch {
            expected: expected_hex.to_owned(),
            actual: actual_hex,
        })
    }
}

/// Extracts a single file by name from an in-memory tar.gz archive.
fn extract_file_from_tar_gz(
    archive_bytes: &[u8],
    target_filename: &str,
) -> Result<Vec<u8>, DownloadError> {
    use std::io::Read;

    let gz = flate2::read::GzDecoder::new(archive_bytes);
    let mut tar = tar::Archive::new(gz);

    for entry_result in tar
        .entries()
        .map_err(|e| DownloadError::Extraction(e.to_string()))?
    {
        let mut entry = entry_result.map_err(|e| DownloadError::Extraction(e.to_string()))?;
        let path = entry
            .path()
            .map_err(|e| DownloadError::Extraction(e.to_string()))?;

        let matches = path.file_name().is_some_and(|name| name == target_filename);

        if matches {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| DownloadError::Extraction(e.to_string()))?;
            return Ok(buf);
        }
    }

    Err(DownloadError::Extraction(format!(
        "{target_filename} not found in archive"
    )))
}

async fn write_with_parent_dirs(target: &Path, data: &[u8]) -> Result<(), std::io::Error> {
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(target, data).await
}

async fn file_exists(path: &Path) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}

fn sqlite_vec_asset_pattern() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "loadable-windows-x86_64"
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "loadable-linux-x86_64"
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "loadable-linux-aarch64"
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "loadable-macos-x86_64"
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "loadable-macos-aarch64"
    }
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
        if let Ok(value) = std::env::var(var_name)
            && !value.trim().is_empty()
        {
            candidates.insert(0, PathBuf::from(value));
        }
    }
    candidates
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut ordered = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        ordered.push(cwd);
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        ordered.push(exe_dir.to_path_buf());
        if let Some(parent) = exe_dir.parent() {
            ordered.push(parent.to_path_buf());
            if let Some(grand_parent) = parent.parent() {
                ordered.push(grand_parent.to_path_buf());
            }
        }
    }

    if let Ok(install_root) = std::env::var("GELI_INSTALL_ROOT")
        && !install_root.trim().is_empty()
    {
        ordered.push(PathBuf::from(install_root));
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

        let copied = seed_from_candidates(&target, std::slice::from_ref(&source))
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
