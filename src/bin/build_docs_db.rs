use geli_shell::shell::config::ShellConfig;
use reqwest::StatusCode;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const EMBEDDING_DIMENSIONS: usize = 768;

#[derive(Debug, thiserror::Error)]
enum IngestError {
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),

    #[error("missing argument value for {0}")]
    MissingArgValue(String),

    #[error("docs directory not found: {0}")]
    DocsDirMissing(String),

    #[error("no markdown files found in docs directory: {0}")]
    NoMarkdownFiles(String),

    #[error("no embeddings were generated successfully")]
    NoEmbeddingsGenerated,

    #[error("failed to read or write files: {0}")]
    Io(#[from] std::io::Error),

    #[error("sqlite error: {0}")]
    Sql(#[from] rusqlite::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error(
        "sqlite-vec extension could not be loaded. tried: {attempts:?}, last error: {last_error}"
    )]
    SqliteVecLoadFailed {
        attempts: Vec<String>,
        last_error: String,
    },
}

#[derive(Debug, Clone)]
struct IngestOptions {
    docs_dir: PathBuf,
    db_path: PathBuf,
    sqlite_vec_path: Option<String>,
    batch_size: usize,
    model: String,
    ollama_url: String,
}

impl IngestOptions {
    fn from_env_and_args() -> Result<Self, IngestError> {
        let cwd = std::env::current_dir()?;
        let mut docs_dir = cwd.join("docs").join("kb");
        let mut db_path = ShellConfig::assistant_docs_db_path();
        let mut sqlite_vec_path = std::env::var("GELI_SQLITE_VEC_PATH")
            .ok()
            .filter(|raw| !raw.trim().is_empty());
        let mut batch_size = 16usize;
        let mut model =
            std::env::var("GELI_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_owned());
        let mut ollama_url = std::env::var("GELI_OLLAMA_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:11434".to_owned());

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--docs-dir" => {
                    docs_dir = PathBuf::from(next_arg_value(&mut args, "--docs-dir")?);
                }
                "--db-path" => {
                    db_path = PathBuf::from(next_arg_value(&mut args, "--db-path")?);
                }
                "--sqlite-vec" => {
                    sqlite_vec_path = Some(next_arg_value(&mut args, "--sqlite-vec")?);
                }
                "--batch-size" => {
                    let raw = next_arg_value(&mut args, "--batch-size")?;
                    batch_size = raw.parse::<usize>().map_err(|_| {
                        IngestError::InvalidArgs(format!(
                            "--batch-size must be a positive integer, got '{raw}'"
                        ))
                    })?;
                    if batch_size == 0 {
                        return Err(IngestError::InvalidArgs(
                            "--batch-size must be greater than zero".to_owned(),
                        ));
                    }
                }
                "--model" => {
                    model = next_arg_value(&mut args, "--model")?;
                }
                "--ollama-url" => {
                    ollama_url = next_arg_value(&mut args, "--ollama-url")?;
                }
                unknown => {
                    return Err(IngestError::InvalidArgs(format!(
                        "unknown argument '{unknown}' (use --help for supported flags)"
                    )));
                }
            }
        }

        if sqlite_vec_path
            .as_deref()
            .map(|p| p.trim().is_empty())
            .unwrap_or(true)
        {
            sqlite_vec_path = resolve_default_sqlite_vec_path();
        }

        if !docs_dir.exists() {
            return Err(IngestError::DocsDirMissing(
                docs_dir.to_string_lossy().into_owned(),
            ));
        }

        Ok(Self {
            docs_dir,
            db_path,
            sqlite_vec_path,
            batch_size,
            model,
            ollama_url,
        })
    }
}

fn next_arg_value(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, IngestError> {
    args.next()
        .ok_or_else(|| IngestError::MissingArgValue(flag.to_owned()))
}

fn print_usage() {
    let default_db_path = ShellConfig::assistant_docs_db_path();
    println!(
        "Usage:
  cargo run --bin build_docs_db -- [options]

Options:
  --docs-dir <path>      Markdown docs directory (default: ./docs/kb)
  --db-path <path>       SQLite output file (default: {})
  --sqlite-vec <path>    sqlite-vec extension path (or use GELI_SQLITE_VEC_PATH)
  --batch-size <n>       Embedding batch size (default: 16)
  --model <name>         Embedding model (default: nomic-embed-text)
  --ollama-url <url>     Local embedding endpoint (default: http://127.0.0.1:11434)",
        default_db_path.display()
    );
}

#[derive(Debug, Clone)]
struct SourceDocument {
    source: String,
    content: String,
}

#[derive(Debug, Clone)]
struct TextChunk {
    id: String,
    source: String,
    text: String,
}

#[derive(Debug, Clone)]
struct EmbeddedChunk {
    chunk: TextChunk,
    embedding: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeadingLevel {
    H1,
    H2,
}

#[derive(Debug, Clone)]
struct SectionHeading {
    level: HeadingLevel,
    title: String,
}

#[derive(Debug, thiserror::Error)]
enum EmbedError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("embedding endpoint '{endpoint}' returned status {status}")]
    HttpStatus {
        endpoint: String,
        status: StatusCode,
    },

    #[error("embedding response mismatch: {0}")]
    InvalidResponse(String),
}

struct EmbeddingClient {
    http: reqwest::Client,
    base_url: String,
    model: String,
}

impl EmbeddingClient {
    fn new(base_url: String, model: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
            model,
        }
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError> {
        let endpoint = format!("{}/api/embed", self.base_url);
        let body = BatchEmbedRequest {
            model: &self.model,
            input: texts.iter().map(String::as_str).collect(),
        };

        let response = self.http.post(&endpoint).json(&body).send().await?;
        if !response.status().is_success() {
            return Err(EmbedError::HttpStatus {
                endpoint,
                status: response.status(),
            });
        }

        let payload: BatchEmbedResponse = response.json().await?;
        if payload.embeddings.len() != texts.len() {
            return Err(EmbedError::InvalidResponse(format!(
                "expected {} embeddings, got {}",
                texts.len(),
                payload.embeddings.len()
            )));
        }
        Ok(payload.embeddings)
    }

    async fn embed_single(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        let endpoint = format!("{}/api/embeddings", self.base_url);
        let body = SingleEmbedRequest {
            model: &self.model,
            prompt: text,
        };

        let response = self.http.post(&endpoint).json(&body).send().await?;
        if !response.status().is_success() {
            return Err(EmbedError::HttpStatus {
                endpoint,
                status: response.status(),
            });
        }

        let payload: SingleEmbedResponse = response.json().await?;
        Ok(payload.embedding)
    }
}

#[derive(Debug, Serialize)]
struct BatchEmbedRequest<'a> {
    model: &'a str,
    input: Vec<&'a str>,
}

#[derive(Debug, Deserialize)]
struct BatchEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Serialize)]
struct SingleEmbedRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Debug, Deserialize)]
struct SingleEmbedResponse {
    embedding: Vec<f32>,
}

#[tokio::main]
async fn main() -> Result<(), IngestError> {
    let options = IngestOptions::from_env_and_args()?;
    println!(
        "INFO: scanning markdown docs in '{}'",
        options.docs_dir.to_string_lossy()
    );

    let documents = read_markdown_documents(&options.docs_dir)?;
    if documents.is_empty() {
        return Err(IngestError::NoMarkdownFiles(
            options.docs_dir.to_string_lossy().into_owned(),
        ));
    }

    let chunks = build_chunks(&documents);
    println!(
        "INFO: loaded {} files and built {} chunks",
        documents.len(),
        chunks.len()
    );

    let client = EmbeddingClient::new(options.ollama_url.clone(), options.model.clone());
    let embedded = generate_embeddings(&client, chunks, options.batch_size).await;
    if embedded.is_empty() {
        return Err(IngestError::NoEmbeddingsGenerated);
    }

    persist_docs_db(&options, &embedded)?;
    println!(
        "INFO: docs.db generated at '{}' with {} vectors",
        options.db_path.to_string_lossy(),
        embedded.len()
    );
    Ok(())
}

fn read_markdown_documents(docs_dir: &Path) -> Result<Vec<SourceDocument>, IngestError> {
    let mut stack = vec![docs_dir.to_path_buf()];
    let mut docs = Vec::new();

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if !file_type.is_file() || !is_markdown_file(&path) {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let source = path
                .strip_prefix(docs_dir)
                .unwrap_or(path.as_path())
                .to_string_lossy()
                .replace('\\', "/");

            docs.push(SourceDocument { source, content });
        }
    }

    docs.sort_by(|left, right| left.source.cmp(&right.source));
    Ok(docs)
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

fn build_chunks(documents: &[SourceDocument]) -> Vec<TextChunk> {
    let mut chunks = Vec::new();
    for document in documents {
        chunks.extend(chunk_markdown_document(document));
    }
    chunks
}

fn chunk_markdown_document(document: &SourceDocument) -> Vec<TextChunk> {
    let mut chunks = Vec::new();
    let mut active_h1: Option<String> = None;
    let mut current_heading: Option<SectionHeading> = None;
    let mut body_buffer = String::new();
    let mut preface = String::new();
    let mut saw_heading = false;

    for line in document.content.lines() {
        if let Some(next_heading) = parse_chunk_heading(line) {
            if let Some(previous_heading) = current_heading.take() {
                push_chunk(
                    &mut chunks,
                    &document.source,
                    active_h1.as_deref(),
                    &previous_heading,
                    &body_buffer,
                );
            }

            body_buffer.clear();
            if !saw_heading && !preface.trim().is_empty() {
                body_buffer.push_str(preface.trim_end());
                body_buffer.push_str("\n\n");
            }

            if matches!(next_heading.level, HeadingLevel::H1) {
                active_h1 = Some(next_heading.title.clone());
            }

            current_heading = Some(next_heading);
            saw_heading = true;
            continue;
        }

        if !saw_heading {
            preface.push_str(line);
            preface.push('\n');
            continue;
        }

        body_buffer.push_str(line);
        body_buffer.push('\n');
    }

    if let Some(last_heading) = current_heading.take() {
        push_chunk(
            &mut chunks,
            &document.source,
            active_h1.as_deref(),
            &last_heading,
            &body_buffer,
        );
    }

    if chunks.is_empty() && !document.content.trim().is_empty() {
        let fallback_text = format!(
            "# Documento: {}\n\n{}",
            document.source,
            document.content.trim()
        );
        chunks.push(TextChunk {
            id: chunk_id(&document.source, &fallback_text),
            source: document.source.clone(),
            text: fallback_text,
        });
    }

    chunks
}

fn parse_chunk_heading(line: &str) -> Option<SectionHeading> {
    if let Some(value) = line.strip_prefix("# ") {
        return Some(SectionHeading {
            level: HeadingLevel::H1,
            title: value.trim().to_owned(),
        });
    }

    if let Some(value) = line.strip_prefix("## ") {
        return Some(SectionHeading {
            level: HeadingLevel::H2,
            title: value.trim().to_owned(),
        });
    }

    None
}

fn push_chunk(
    out: &mut Vec<TextChunk>,
    source: &str,
    active_h1: Option<&str>,
    heading: &SectionHeading,
    body: &str,
) {
    let text = assemble_chunk_text(active_h1, heading, body);
    let id = chunk_id(source, &text);
    out.push(TextChunk {
        id,
        source: source.to_owned(),
        text,
    });
}

fn assemble_chunk_text(active_h1: Option<&str>, heading: &SectionHeading, body: &str) -> String {
    let body = body.trim();

    match heading.level {
        HeadingLevel::H1 => {
            if body.is_empty() {
                format!("# {}", heading.title)
            } else {
                format!("# {}\n\n{body}", heading.title)
            }
        }
        HeadingLevel::H2 => {
            let mut out = String::new();
            if let Some(h1) = active_h1 {
                out.push_str("# ");
                out.push_str(h1);
                out.push_str("\n\n");
            }

            out.push_str("## ");
            out.push_str(&heading.title);
            if !body.is_empty() {
                out.push_str("\n\n");
                out.push_str(body);
            }
            out
        }
    }
}

fn chunk_id(source: &str, text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    hasher.update([0_u8]);
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn generate_embeddings(
    client: &EmbeddingClient,
    chunks: Vec<TextChunk>,
    batch_size: usize,
) -> Vec<EmbeddedChunk> {
    let mut embedded = Vec::new();
    let effective_batch_size = batch_size.max(1);

    for batch in chunks.chunks(effective_batch_size) {
        let texts: Vec<String> = batch.iter().map(|chunk| chunk.text.clone()).collect();

        match client.embed_batch(&texts).await {
            Ok(vectors) => {
                append_valid_embeddings(&mut embedded, batch, vectors);
            }
            Err(error) => {
                eprintln!("WARN: batch embedding failed ({error}); retrying one-by-one");
                for chunk in batch {
                    match client.embed_single(&chunk.text).await {
                        Ok(vector) => {
                            append_valid_embeddings(
                                &mut embedded,
                                std::slice::from_ref(chunk),
                                vec![vector],
                            );
                        }
                        Err(chunk_error) => {
                            eprintln!(
                                "WARN: embedding failed for chunk '{}' from '{}': {}",
                                chunk.id, chunk.source, chunk_error
                            );
                        }
                    }
                }
            }
        }
    }

    embedded
}

fn append_valid_embeddings(
    out: &mut Vec<EmbeddedChunk>,
    chunks: &[TextChunk],
    vectors: Vec<Vec<f32>>,
) {
    if chunks.len() != vectors.len() {
        eprintln!(
            "WARN: embedding count mismatch for batch (chunks={}, embeddings={})",
            chunks.len(),
            vectors.len()
        );
        return;
    }

    for (chunk, embedding) in chunks.iter().cloned().zip(vectors) {
        if embedding.len() != EMBEDDING_DIMENSIONS {
            eprintln!(
                "WARN: embedding dimension mismatch for chunk '{}' from '{}': got {}, expected {}",
                chunk.id,
                chunk.source,
                embedding.len(),
                EMBEDDING_DIMENSIONS
            );
            continue;
        }
        assert_embedding_dimensions(&embedding);
        out.push(EmbeddedChunk { chunk, embedding });
    }
}

fn assert_embedding_dimensions(embedding: &[f32]) {
    assert_eq!(
        embedding.len(),
        EMBEDDING_DIMENSIONS,
        "embedding dimension must be exactly {EMBEDDING_DIMENSIONS}"
    );
}

fn persist_docs_db(options: &IngestOptions, embedded: &[EmbeddedChunk]) -> Result<(), IngestError> {
    if let Some(parent) = options.db_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let mut conn = Connection::open(&options.db_path)?;
    let loaded_extension = load_sqlite_vec_extension(&conn, options.sqlite_vec_path.as_deref())?;
    println!("INFO: loaded sqlite-vec extension from '{loaded_extension}'");

    conn.execute_batch(
        "
        DROP TABLE IF EXISTS docs_metadata;
        DROP TABLE IF EXISTS vec_docs;

        CREATE TABLE docs_metadata (
            id TEXT PRIMARY KEY,
            fuente TEXT NOT NULL,
            texto_completo TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE vec_docs USING vec0(
            id TEXT,
            embedding float[768]
        );
        ",
    )?;

    let tx = conn.transaction()?;
    for item in embedded {
        assert_embedding_dimensions(&item.embedding);
        let embedding_literal = embedding_to_json_array(&item.embedding);

        tx.execute(
            "INSERT INTO docs_metadata (id, fuente, texto_completo) VALUES (?1, ?2, ?3)",
            params![item.chunk.id, item.chunk.source, item.chunk.text],
        )?;

        tx.execute(
            "INSERT INTO vec_docs (id, embedding) VALUES (?1, ?2)",
            params![item.chunk.id, embedding_literal],
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn load_sqlite_vec_extension(
    conn: &Connection,
    configured_path: Option<&str>,
) -> Result<String, IngestError> {
    unsafe {
        conn.load_extension_enable()?;
    }

    let mut attempts = Vec::new();
    let candidates = sqlite_vec_candidates(configured_path);
    let mut last_error = "no candidate executed".to_owned();

    for candidate in candidates {
        attempts.push(candidate.clone());
        let result = unsafe { conn.load_extension(&candidate, None) };
        match result {
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
    Err(IngestError::SqliteVecLoadFailed {
        attempts,
        last_error,
    })
}

fn sqlite_vec_candidates(configured_path: Option<&str>) -> Vec<String> {
    if let Some(path) = configured_path {
        return vec![normalize_path_str(path)];
    }

    #[cfg(target_os = "windows")]
    {
        vec![
            "vec0.dll".to_owned(),
            "sqlite_vec.dll".to_owned(),
            "sqlite-vec.dll".to_owned(),
            "vec0".to_owned(),
            "sqlite_vec".to_owned(),
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec![
            "vec0".to_owned(),
            "vec0.so".to_owned(),
            "sqlite_vec.so".to_owned(),
            "sqlite-vec.so".to_owned(),
        ]
    }

    #[cfg(target_os = "macos")]
    {
        vec![
            "vec0".to_owned(),
            "vec0.dylib".to_owned(),
            "sqlite_vec.dylib".to_owned(),
            "sqlite-vec.dylib".to_owned(),
        ]
    }
}

fn resolve_default_sqlite_vec_path() -> Option<String> {
    if let Some(home) = dirs::home_dir() {
        let ext = if cfg!(target_os = "windows") {
            "dll"
        } else {
            "so"
        };
        let candidate = home
            .join(".config")
            .join("geliShell")
            .join("models")
            .join(format!("vec0.{ext}"));

        if candidate.exists() {
            return Some(normalize_path(&candidate));
        }
    }
    None
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_path_str(path: &str) -> String {
    path.replace('\\', "/")
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
