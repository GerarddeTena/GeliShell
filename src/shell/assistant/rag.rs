use crate::shell::config::ShellConfig;
use crate::t;
use reqwest::StatusCode;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

const EMBEDDING_DIMENSIONS: usize = 768;

#[derive(Debug, thiserror::Error)]
pub enum RagError {
    #[error(
        "vector knowledge base or DLL not found — run geli-update to initialize the assistant. {details}"
    )]
    KnowledgeBaseUnavailable { details: String },

    #[error("failed to fetch embedding for retrieval query: {0}")]
    Http(#[from] reqwest::Error),

    #[error("sqlite retrieval failed: {0}")]
    Sql(#[from] rusqlite::Error),

    #[error("embedding endpoint '{endpoint}' returned status {status}\n\nDetails: {details}")]
    HttpStatus {
        endpoint: String,
        status: StatusCode,
        details: String,
    },

    #[error("embedding model returned {got} dimensions, expected {expected}")]
    InvalidEmbeddingDimensions { expected: usize, got: usize },

    #[error(
        "sqlite-vec extension could not be loaded. tried: {attempts:?}, last error: {last_error}"
    )]
    SqliteVecLoadFailed {
        attempts: Vec<String>,
        last_error: String,
    },

    #[error("background retrieval task failed: {0}")]
    Join(#[from] tokio::task::JoinError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RagChunk {
    pub path: String,
    pub text: String,
    pub distance: f32,
}

#[derive(Debug)]
pub struct RagEngine {
    models_dir: PathBuf,
    db_path: PathBuf,
    sqlite_vec_path: Option<String>,
    embedding_model: String,
    ollama_url: String,
    http: reqwest::Client,
}

impl RagEngine {
    pub fn new(models_dir: PathBuf) -> Self {
        let db_path = ShellConfig::assistant_docs_db_path();
        let sqlite_vec_path = resolve_sqlite_vec_path(&models_dir);
        let embedding_model =
            std::env::var("GELI_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_owned());
        let ollama_url = std::env::var("GELI_OLLAMA_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:11434".to_owned());

        Self {
            models_dir,
            db_path,
            sqlite_vec_path,
            embedding_model,
            ollama_url,
            http: reqwest::Client::new(),
        }
    }

    pub async fn clear_cache(&self) {}

    pub async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<RagChunk>, RagError> {
        if query.trim().is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let query_embedding = self.embed_query(query).await?;
        let query_vector = embedding_to_json_array(&query_embedding);
        let db_path = self.db_path.clone();
        let models_dir = self.models_dir.clone();
        let sqlite_vec_path = self.sqlite_vec_path.clone();

        let rows = tokio::task::spawn_blocking(move || {
            search_vector_db(
                &db_path,
                &models_dir,
                sqlite_vec_path.as_deref(),
                &query_vector,
                limit,
            )
        })
        .await??;

        Ok(rows
            .into_iter()
            .map(|row| RagChunk {
                path: row.source,
                text: row.text,
                distance: row.distance,
            })
            .collect())
    }

    pub async fn retrieve_context(&self, query: &str, limit: usize) -> Result<String, RagError> {
        let chunks = self.retrieve(query, limit).await?;
        Ok(format_chunks_for_prompt(&chunks))
    }

    async fn embed_query(&self, query: &str) -> Result<Vec<f32>, RagError> {
        let endpoint = format!("{}/api/embeddings", self.ollama_url.trim_end_matches('/'));
        let response = self
            .http
            .post(&endpoint)
            .json(&QueryEmbedRequest {
                model: &self.embedding_model,
                prompt: query,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let details = response
                .text()
                .await
                .unwrap_or_else(|_| "(unable to read response body)".to_string());
            
            return Err(RagError::HttpStatus {
                endpoint,
                status,
                details,
            });
        }

        let payload: QueryEmbedResponse = response.json().await?;
        if payload.embedding.len() != EMBEDDING_DIMENSIONS {
            return Err(RagError::InvalidEmbeddingDimensions {
                expected: EMBEDDING_DIMENSIONS,
                got: payload.embedding.len(),
            });
        }
        Ok(payload.embedding)
    }
}

fn format_chunks_for_prompt(chunks: &[RagChunk]) -> String {
    if chunks.is_empty() {
        return t!("assistant.rag.no_context");
    }

    let source_label = t!("assistant.rag.chunk_source_label");
    let distance_label = t!("assistant.rag.chunk_distance_label");

    chunks
        .iter()
        .map(|chunk| {
            format!(
                "- {}: {} | {}: {:.4}\n{}",
                source_label,
                chunk.path,
                distance_label,
                chunk.distance,
                chunk.text.trim()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[derive(Debug, Serialize)]
struct QueryEmbedRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Debug, Deserialize)]
struct QueryEmbedResponse {
    embedding: Vec<f32>,
}

#[derive(Debug)]
struct DbMatch {
    source: String,
    text: String,
    distance: f32,
}

fn search_vector_db(
    db_path: &Path,
    models_dir: &Path,
    configured_vec_path: Option<&str>,
    query_vector: &str,
    limit: usize,
) -> Result<Vec<DbMatch>, RagError> {
    if !db_path.exists() {
        return Err(RagError::KnowledgeBaseUnavailable {
            details: format!("Missing docs.db at '{}'.", db_path.display()),
        });
    }

    let conn = Connection::open(db_path)?;
    match load_sqlite_vec_extension(&conn, models_dir, configured_vec_path) {
        Ok(_) => {}
        Err(RagError::SqliteVecLoadFailed {
            attempts,
            last_error,
        }) => {
            return Err(RagError::KnowledgeBaseUnavailable {
                details: format!(
                    "sqlite-vec load failed. tried: {:?}; last error: {}",
                    attempts, last_error
                ),
            });
        }
        Err(other) => return Err(other),
    }

    ensure_required_schema(&conn, db_path)?;

    let mut stmt = conn.prepare(
        "
        SELECT
            m.fuente,
            m.texto_completo,
            CAST(vec_distance_cosine(v.embedding, ?1) AS REAL) AS distance
        FROM vec_docs v
        JOIN docs_metadata m ON m.id = v.id
        ORDER BY distance ASC
        LIMIT ?2
        ",
    )?;

    let rows = stmt.query_map(params![query_vector, limit as i64], |row| {
        Ok(DbMatch {
            source: row.get::<_, String>(0)?,
            text: row.get::<_, String>(1)?,
            distance: row.get::<_, f32>(2)?,
        })
    })?;

    let mut matches = Vec::new();
    for row in rows {
        matches.push(row?);
    }
    Ok(matches)
}

fn ensure_required_schema(conn: &Connection, db_path: &Path) -> Result<(), RagError> {
    let has_docs_metadata = table_exists(conn, "docs_metadata")?;
    let has_vec_docs = table_exists(conn, "vec_docs")?;
    if has_docs_metadata && has_vec_docs {
        return Ok(());
    }

    Err(RagError::KnowledgeBaseUnavailable {
        details: format!(
            "Invalid docs.db schema at '{}'. Required tables vec_docs and docs_metadata were not found.",
            db_path.display()
        ),
    })
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool, RagError> {
    let mut stmt = conn.prepare(
        "
        SELECT 1
        FROM sqlite_master
        WHERE type IN ('table', 'view')
          AND name = ?1
        LIMIT 1
        ",
    )?;
    let mut rows = stmt.query(params![table])?;
    Ok(rows.next()?.is_some())
}

fn load_sqlite_vec_extension(
    conn: &Connection,
    models_dir: &Path,
    configured_path: Option<&str>,
) -> Result<String, RagError> {
    unsafe {
        conn.load_extension_enable()?;
    }

    let mut attempts = Vec::new();
    let mut last_error = "no extension candidate executed".to_owned();

    for candidate in sqlite_vec_candidates(models_dir, configured_path) {
        attempts.push(candidate.clone());
        let load_result = unsafe { conn.load_extension(&candidate, None) };
        match load_result {
            Ok(()) => {
                conn.load_extension_disable()?;
                return Ok(candidate);
            }
            Err(error) => {
                last_error = error.to_string();
            }
        }
    }

    conn.load_extension_disable()?;
    Err(RagError::SqliteVecLoadFailed {
        attempts,
        last_error,
    })
}

fn sqlite_vec_candidates(models_dir: &Path, configured_path: Option<&str>) -> Vec<String> {
    if let Some(path) = configured_path {
        // Return the user-configured path as-is — do not normalize.
        // The user (or installer) chose this path deliberately; altering it
        // would silently break explicit GELI_SQLITE_VEC_PATH overrides.
        return vec![path.to_owned()];
    }

    #[cfg(target_os = "windows")]
    let library_names = [
        "vec0.dll",
        "sqlite_vec.dll",
        "sqlite-vec.dll",
        "vec0",
        "sqlite_vec",
    ];
    #[cfg(target_os = "linux")]
    let library_names = ["vec0", "vec0.so", "sqlite_vec.so", "sqlite-vec.so"];
    #[cfg(target_os = "macos")]
    let library_names = ["vec0", "vec0.dylib", "sqlite_vec.dylib", "sqlite-vec.dylib"];

    let mut set = BTreeSet::new();
    for library_name in library_names {
        set.insert(normalize_path(&models_dir.join(library_name)));
        set.insert(library_name.to_owned());
    }
    set.into_iter().collect()
}

fn resolve_sqlite_vec_path(models_dir: &Path) -> Option<String> {
    if let Ok(raw) = std::env::var("GELI_SQLITE_VEC_PATH") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.exists() {
                return Some(normalize_path(&candidate));
            }
        }
    }

    // Platform-correct extension name
    #[cfg(target_os = "windows")]
    let ext_name = "vec0.dll";
    #[cfg(target_os = "linux")]
    let ext_name = "vec0.so";
    #[cfg(target_os = "macos")]
    let ext_name = "vec0.dylib";

    if let Some(home) = dirs::home_dir() {
        let candidate = home
            .join(".config")
            .join("geliShell")
            .join("models")
            .join(ext_name);
        if candidate.exists() {
            return Some(normalize_path(&candidate));
        }
    }

    let models_candidate = models_dir.join(ext_name);
    if models_candidate.exists() {
        return Some(normalize_path(&models_candidate));
    }

    None
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn embedding_to_json_array(embedding: &[f32]) -> String {
    let mut out = String::with_capacity(embedding.len() * 12 + 2);
    out.push('[');
    for (index, value) in embedding.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&format!("{value:.8}"));
    }
    out.push(']');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn embedding_to_json_array_is_stable() {
        let encoded = embedding_to_json_array(&[1.0, 2.5, -3.0]);
        assert_eq!(encoded, "[1.00000000,2.50000000,-3.00000000]");
    }

    #[test]
    fn candidates_prefer_configured_path() {
        let models_dir = std::env::temp_dir().join("geli_shell_rag_candidates");
        // configured_path is returned as-is (no normalization) so the caller's
        // explicit path override is preserved exactly as provided.
        let configured = "C:\\custom\\vec0.dll";
        let candidates = sqlite_vec_candidates(&models_dir, Some(configured));
        assert_eq!(candidates, vec!["C:\\custom\\vec0.dll".to_owned()]);
    }

    #[test]
    fn missing_docs_db_returns_user_facing_error() {
        let models_dir = unique_test_dir("rag_models");
        let db_path = unique_test_dir("rag_docs").join("docs.db");
        let result = search_vector_db(&db_path, &models_dir, None, "[0.0,0.0]", 3);

        let Err(RagError::KnowledgeBaseUnavailable { details }) = result else {
            panic!("expected KnowledgeBaseUnavailable when docs.db is missing");
        };
        assert!(details.contains("Missing docs.db"));
    }

    #[test]
    fn format_chunks_for_prompt_is_non_empty_without_matches() {
        let rendered = format_chunks_for_prompt(&[]);
        assert!(rendered.contains("No RAG context retrieved"));
    }

    #[test]
    fn format_chunks_for_prompt_includes_source_and_text() {
        let rendered = format_chunks_for_prompt(&[RagChunk {
            path: "docs/kb/guardrail.md".to_owned(),
            text: "Use canonical commands only.".to_owned(),
            distance: 0.42,
        }]);

        assert!(rendered.contains("docs/kb/guardrail.md"));
        assert!(rendered.contains("Use canonical commands only."));
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_millis();
        std::env::temp_dir().join(format!("geli_shell_{prefix}_{millis}"))
    }
}
