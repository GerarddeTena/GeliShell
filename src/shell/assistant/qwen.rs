use crate::shell::config::AssistantModelVariant;
use futures_util::StreamExt;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

#[derive(Debug, thiserror::Error)]
pub enum QwenError {
    #[error("failed to access model files: {0}")]
    Io(#[from] std::io::Error),

    #[error("network request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("remote download returned status {0}")]
    HttpStatus(reqwest::StatusCode),

    #[error("model file is not GGUF (missing header)")]
    InvalidGgufHeader,

    #[error("model is not loaded in memory")]
    ModelNotLoaded,

    #[error("background task join failed: {0}")]
    Join(#[from] tokio::task::JoinError),
}

#[derive(Debug, Clone)]
pub enum BootstrapEvent {
    CheckingModel {
        path: String,
    },
    ExistingModelFound {
        path: String,
        size_bytes: u64,
    },
    Downloading {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    VerifyingModel,
    ModelLoaded {
        path: String,
        size_bytes: u64,
    },
    Failed {
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub struct ModelArtifact {
    pub variant: AssistantModelVariant,
    pub path: PathBuf,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
struct LoadedModel {
    artifact: ModelArtifact,
    last_used: Instant,
}

pub struct QwenRuntime {
    models_dir: PathBuf,
    active: Option<LoadedModel>,
}

impl QwenRuntime {
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            active: None,
        }
    }

    pub fn unload(&mut self) {
        self.active = None;
    }

    pub fn maybe_unload_idle(&mut self, idle_after_secs: u64) -> bool {
        if idle_after_secs == 0 {
            self.unload();
            return true;
        }

        let Some(active) = &self.active else {
            return false;
        };

        if active.last_used.elapsed() >= Duration::from_secs(idle_after_secs) {
            self.unload();
            return true;
        }
        false
    }

    pub async fn ensure_ready_and_load(
        &mut self,
        variant: AssistantModelVariant,
        progress: mpsc::UnboundedSender<BootstrapEvent>,
    ) -> Result<ModelArtifact, QwenError> {
        let artifact = match self.ensure_model_file(variant, &progress).await {
            Ok(artifact) => artifact,
            Err(error) => {
                let _ = progress.send(BootstrapEvent::Failed {
                    reason: error.to_string(),
                });
                return Err(error);
            }
        };

        if let Err(error) = self.load_model(artifact.clone()).await {
            let _ = progress.send(BootstrapEvent::Failed {
                reason: error.to_string(),
            });
            return Err(error);
        }

        let _ = progress.send(BootstrapEvent::ModelLoaded {
            path: artifact.path.to_string_lossy().replace('\\', "/"),
            size_bytes: artifact.size_bytes,
        });
        Ok(artifact)
    }

    pub async fn generate(&mut self, prompt: String) -> Result<String, QwenError> {
        let Some(active) = &mut self.active else {
            return Err(QwenError::ModelNotLoaded);
        };
        active.last_used = Instant::now();
        let model = active.artifact.variant.as_str().to_owned();

        let generated =
            tokio::task::spawn_blocking(move || synthesize_response(&model, &prompt)).await?;

        Ok(generated)
    }

    async fn ensure_model_file(
        &self,
        variant: AssistantModelVariant,
        progress: &mpsc::UnboundedSender<BootstrapEvent>,
    ) -> Result<ModelArtifact, QwenError> {
        let final_path = self.models_dir.join(model_filename(variant));
        let normalized_path = final_path.to_string_lossy().replace('\\', "/");
        let _ = progress.send(BootstrapEvent::CheckingModel {
            path: normalized_path.clone(),
        });

        if let Ok(metadata) = tokio::fs::metadata(&final_path).await {
            if metadata.len() > 0 {
                let _ = progress.send(BootstrapEvent::ExistingModelFound {
                    path: normalized_path,
                    size_bytes: metadata.len(),
                });
                return Ok(ModelArtifact {
                    variant,
                    path: final_path,
                    size_bytes: metadata.len(),
                });
            }
        }

        tokio::fs::create_dir_all(&self.models_dir).await?;

        let tmp_path = final_path.with_extension("download.part");
        if let Err(error) = download_model(variant, &tmp_path, progress).await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(error);
        }

        let _ = progress.send(BootstrapEvent::VerifyingModel);
        if let Err(error) = verify_gguf_magic(tmp_path.clone()).await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(error);
        }

        if tokio::fs::metadata(&final_path).await.is_ok() {
            tokio::fs::remove_file(&final_path).await?;
        }
        tokio::fs::rename(&tmp_path, &final_path).await?;

        let metadata = tokio::fs::metadata(&final_path).await?;
        Ok(ModelArtifact {
            variant,
            path: final_path,
            size_bytes: metadata.len(),
        })
    }

    async fn load_model(&mut self, artifact: ModelArtifact) -> Result<(), QwenError> {
        let validate_path = artifact.path.clone();
        tokio::task::spawn_blocking(move || verify_gguf_magic_blocking(&validate_path)).await??;

        self.active = Some(LoadedModel {
            artifact,
            last_used: Instant::now(),
        });
        Ok(())
    }
}

async fn download_model(
    variant: AssistantModelVariant,
    tmp_path: &Path,
    progress: &mpsc::UnboundedSender<BootstrapEvent>,
) -> Result<(), QwenError> {
    let client = reqwest::Client::new();
    let response = client.get(model_url(variant)).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(QwenError::HttpStatus(status));
    }

    let total_bytes = response.content_length();
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(tmp_path).await?;
    let mut downloaded_bytes = 0u64;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        downloaded_bytes += chunk.len() as u64;

        let _ = progress.send(BootstrapEvent::Downloading {
            downloaded_bytes,
            total_bytes,
        });
    }

    file.flush().await?;
    Ok(())
}

async fn verify_gguf_magic(path: PathBuf) -> Result<(), QwenError> {
    tokio::task::spawn_blocking(move || verify_gguf_magic_blocking(&path)).await??;
    Ok(())
}

fn verify_gguf_magic_blocking(path: &Path) -> Result<(), QwenError> {
    let mut file = std::fs::File::open(path)?;
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic)?;
    if &magic != b"GGUF" {
        return Err(QwenError::InvalidGgufHeader);
    }
    Ok(())
}

fn model_filename(variant: AssistantModelVariant) -> &'static str {
    match variant {
        AssistantModelVariant::Qwen05b => "qwen2.5-0.5b-instruct-q4_k_m.gguf",
        AssistantModelVariant::Qwen15b => "qwen2.5-1.5b-instruct-q4_k_m.gguf",
    }
}

fn model_url(variant: AssistantModelVariant) -> &'static str {
    match variant {
        AssistantModelVariant::Qwen05b => {
            "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf?download=true"
        }
        AssistantModelVariant::Qwen15b => {
            "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf?download=true"
        }
    }
}

fn synthesize_response(_model: &str, prompt: &str) -> String {
    let rag_context = extract_rag_context(prompt).unwrap_or_default();
    let user_action = extract_user_action(prompt).unwrap_or_default();

    if expects_how_to_contract(prompt) {
        let subsystem = extract_how_to_subsystem(prompt);
        let command = select_command_from_context(
            &rag_context,
            subsystem.as_deref(),
            Some(user_action.as_str()),
        )
        .or_else(|| fallback_context_line(&rag_context))
        .unwrap_or_else(|| "contexto_rag_sin_comando".to_owned());

        let explanation = build_how_to_explanation(&rag_context, subsystem.as_deref());
        return format!("EXPLANATION: {explanation}\nCOMMAND: {command}");
    }

    select_command_from_context(&rag_context, None, Some(user_action.as_str()))
        .or_else(|| fallback_context_line(&rag_context))
        .unwrap_or_else(|| "No encontré una solución en el contexto recuperado de RAG.".to_owned())
}

fn expects_how_to_contract(prompt: &str) -> bool {
    // EXPLANATION: and COMMAND: are language-neutral structural tokens present
    // identically in every locale's how_to prompt template. They do NOT appear
    // in the show_me template, making them a safe discriminator.
    prompt.contains("EXPLANATION:") && prompt.contains("COMMAND:")
}

fn extract_rag_context(prompt: &str) -> Option<String> {
    // Primary: language-neutral markers used in all locale prompt templates.
    if let Some(context) = extract_between(prompt, "[CONTEXT]", "[END CONTEXT]") {
        let trimmed = context.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }

    // Legacy fallback: Spanish markers from older builds / handcrafted prompts.
    if let Some(context) = extract_between(prompt, "[CONTEXTO]", "[FIN CONTEXTO]") {
        let trimmed = context.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }

    // Legacy fallback: old inline RAG marker.
    if let Some((_, tail)) = prompt.split_once("[CONTEXTO RECUPERADO DE RAG]") {
        let context = tail
            .split_once("<|im_end|>")
            .map(|(context, _)| context)
            .unwrap_or(tail)
            .trim();
        if !context.is_empty() {
            return Some(context.to_owned());
        }
    }

    None
}

fn extract_between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let (_, tail) = text.split_once(start)?;
    let (inside, _) = tail.split_once(end)?;
    Some(inside)
}

fn extract_how_to_subsystem(prompt: &str) -> Option<String> {
    // Try language-neutral marker first, then legacy Spanish marker.
    let lowercase = prompt.to_ascii_lowercase();
    for marker in &["subsystem:", "subsistema:"] {
        let Some(marker_idx) = lowercase.find(marker) else {
            continue;
        };
        let tail = &prompt[marker_idx + marker.len()..];
        let subsystem = tail
            .split(|ch| ch == ',' || ch == '\n')
            .next()?
            .trim()
            .trim_end_matches('.');
        if !subsystem.is_empty() {
            return Some(subsystem.to_owned());
        }
    }
    None
}

fn extract_user_action(prompt: &str) -> Option<String> {
    let (_, user_block) = prompt.split_once("<|im_start|>user")?;
    let user_block = user_block.trim_start_matches('\n');
    let (action, _) = user_block.split_once("<|im_end|>")?;
    let action = action.trim();
    if action.is_empty() {
        None
    } else {
        Some(action.to_owned())
    }
}

#[derive(Debug, Clone)]
struct CommandCandidate {
    command: String,
    line_label: Option<String>,
    line_index: usize,
}

fn select_command_from_context(
    rag_context: &str,
    subsystem: Option<&str>,
    user_action: Option<&str>,
) -> Option<String> {
    let lines: Vec<&str> = rag_context.lines().map(str::trim).collect();
    if lines.is_empty() {
        return None;
    }

    let candidates = command_candidates(&lines);
    if candidates.is_empty() {
        return None;
    }

    let subsystem_targets = subsystem_labels(subsystem);
    let candidate_pool: Vec<&CommandCandidate> = if subsystem_targets.is_empty() {
        candidates.iter().collect()
    } else {
        let matched: Vec<&CommandCandidate> = candidates
            .iter()
            .filter(|candidate| {
                line_matches_subsystem(candidate.line_label.as_deref(), &subsystem_targets)
            })
            .collect();
        if matched.is_empty() {
            candidates.iter().collect()
        } else {
            matched
        }
    };

    let action_tokens = tokenize(user_action.unwrap_or_default());
    let mut best: Option<(i32, usize, String)> = None;

    for candidate in candidate_pool {
        let start = candidate.line_index.saturating_sub(2);
        let end = usize::min(candidate.line_index + 2, lines.len().saturating_sub(1));
        let window = lines[start..=end].join(" ").to_ascii_lowercase();
        let token_score = action_tokens
            .iter()
            .filter(|token| window.contains(token.as_str()))
            .count() as i32;

        let score = token_score * 5;
        let should_replace = match &best {
            None => true,
            Some((best_score, best_index, _)) => {
                score > *best_score || (score == *best_score && candidate.line_index < *best_index)
            }
        };

        if should_replace {
            best = Some((score, candidate.line_index, candidate.command.clone()));
        }
    }

    best.map(|(_, _, command)| command)
}

fn command_candidates(lines: &[&str]) -> Vec<CommandCandidate> {
    let mut out = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        if line.is_empty() || is_context_metadata_line(line) {
            continue;
        }

        if let Some(command) = extract_command_from_line(line) {
            out.push(CommandCandidate {
                command,
                line_label: label_for_line(line),
                line_index,
            });
        }
    }

    out
}

fn extract_command_from_line(line: &str) -> Option<String> {
    let lowercase = line.to_ascii_lowercase();
    if lowercase.starts_with("## intención:") || lowercase.starts_with("# intención:") {
        return None;
    }

    if let Some(backtick_start) = line.find('`') {
        let rest = &line[backtick_start + 1..];
        if let Some(backtick_end) = rest.find('`') {
            let command = rest[..backtick_end].trim();
            if !command.is_empty() {
                return Some(command.to_owned());
            }
        }
    }

    let (_, tail) = line.split_once(':')?;
    let command = tail.trim().trim_matches('`');
    if looks_command_like(command) {
        Some(command.to_owned())
    } else {
        None
    }
}

fn label_for_line(line: &str) -> Option<String> {
    let lowercase = line.to_ascii_lowercase();
    if lowercase.contains("bash/zsh") {
        Some("bash/zsh".to_owned())
    } else if lowercase.contains("powershell") {
        Some("powershell".to_owned())
    } else if lowercase.contains("fish") {
        Some("fish".to_owned())
    } else if lowercase.contains("cmd") {
        Some("cmd".to_owned())
    } else if lowercase.contains("bash") {
        Some("bash".to_owned())
    } else if lowercase.contains("zsh") {
        Some("zsh".to_owned())
    } else {
        None
    }
}

fn subsystem_labels(subsystem: Option<&str>) -> Vec<&'static str> {
    let Some(subsystem) = subsystem else {
        return Vec::new();
    };
    let normalized = subsystem.trim().to_ascii_lowercase();

    if normalized.contains("power") {
        vec!["powershell"]
    } else if normalized.contains("fish") {
        vec!["fish"]
    } else if normalized.contains("cmd") || normalized.contains("command prompt") {
        vec!["cmd"]
    } else if normalized.contains("zsh") {
        vec!["bash/zsh", "zsh", "bash"]
    } else if normalized.contains("bash") {
        vec!["bash/zsh", "bash", "zsh"]
    } else {
        Vec::new()
    }
}

fn line_matches_subsystem(line_label: Option<&str>, targets: &[&str]) -> bool {
    if targets.is_empty() {
        return true;
    }
    let Some(line_label) = line_label else {
        return false;
    };
    targets.iter().any(|target| *target == line_label)
}

fn fallback_context_line(rag_context: &str) -> Option<String> {
    for line in rag_context.lines().map(str::trim) {
        if line.is_empty() || is_context_metadata_line(line) {
            continue;
        }
        if let Some(intent) = line.strip_prefix("## Intención:") {
            let cleaned = intent.trim();
            if !cleaned.is_empty() {
                return Some(cleaned.to_owned());
            }
            continue;
        }
        return Some(line.trim_matches('`').to_owned());
    }
    None
}

fn is_context_metadata_line(line: &str) -> bool {
    let lowercase = line.to_ascii_lowercase();
    lowercase.starts_with("- source:")          // EN (neutral)
        || lowercase.starts_with("- fuente:")   // ES legacy
        || lowercase.starts_with("cosine distance:") // EN (neutral)
        || lowercase.starts_with("distancia coseno:") // ES legacy
        || lowercase.starts_with("[context]")        // EN (neutral)
        || lowercase.starts_with("[end context]")    // EN (neutral)
        || lowercase.starts_with("[contexto]")       // ES legacy
        || lowercase.starts_with("[fin contexto]") // ES legacy
}

fn looks_command_like(candidate: &str) -> bool {
    let cleaned = candidate.trim();
    if cleaned.is_empty() {
        return false;
    }
    let lowercase = cleaned.to_ascii_lowercase();
    // Reject lines that are clearly metadata or intent headers from the KB.
    // "intención" / "source" / "fuente" are never valid shell commands.
    if lowercase.starts_with("intención")
        || lowercase.starts_with("source")
        || lowercase.starts_with("fuente")
    {
        return false;
    }
    let first_token = cleaned.split_whitespace().next().unwrap_or_default();
    first_token
        .chars()
        .any(|ch| ch.is_ascii_alphanumeric() || "$./\\_-".contains(ch))
}

fn build_how_to_explanation(rag_context: &str, subsystem: Option<&str>) -> String {
    if let Some(intent) = extract_intent_line(rag_context) {
        return crate::t!("assistant.how_to_expl_from_intent", intent = intent);
    }

    if let Some(subsystem) = subsystem {
        return crate::t!(
            "assistant.how_to_expl_from_subsystem",
            subsystem = subsystem
        );
    }

    crate::t!("assistant.how_to_expl_generic")
}

fn extract_intent_line(rag_context: &str) -> Option<String> {
    for line in rag_context.lines().map(str::trim) {
        if let Some(intent) = line.strip_prefix("## Intención:") {
            let cleaned = intent.trim();
            if !cleaned.is_empty() {
                return Some(cleaned.to_owned());
            }
        }
    }
    None
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| token.len() >= 3)
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn variant_paths_are_stable() {
        assert!(model_filename(AssistantModelVariant::Qwen05b).ends_with(".gguf"));
        assert!(model_url(AssistantModelVariant::Qwen15b).contains("huggingface.co"));
    }

    #[test]
    fn gguf_magic_validation_rejects_invalid_header() {
        let path = unique_temp_file("invalid_gguf_header.bin");
        std::fs::write(&path, b"NOPE").unwrap();
        let result = verify_gguf_magic_blocking(&path);
        assert!(matches!(result, Err(QwenError::InvalidGgufHeader)));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn extract_user_action_reads_chatml_user_block() {
        let prompt = "<|im_start|>system\nctx\n<|im_end|>\n<|im_start|>user\nSearch in files\n<|im_end|>\n<|im_start|>assistant";
        let action = extract_user_action(prompt);
        assert_eq!(action.as_deref(), Some("Search in files"));
    }

    #[test]
    fn synthesize_response_extracts_command_from_rag_context() {
        let prompt = "<|im_start|>system\n[CONTEXT]\n## Intención: Buscar texto por patrón\n- Bash/Zsh: `rg \"<patron>\" <ruta_base>`\n- PowerShell: `Select-String -Path \"<ruta_base>\\*\" -Pattern \"<patron>\" -Recurse`\n[END CONTEXT]\n<|im_end|>\n<|im_start|>user\nSearch in files\n<|im_end|>\n<|im_start|>assistant\n";
        let generated = synthesize_response("qwen2.5-1.5b-instruct-q4_k_m", prompt);

        assert_eq!(generated, "rg \"<patron>\" <ruta_base>");
    }

    #[test]
    fn strict_how_to_prompt_produces_two_line_contract() {
        let prompt = "<|im_start|>system\nYou are a strict terminal assistant. Your only purpose is to extract the exact command for subsystem: powershell, based EXCLUSIVELY on the following context.\n[CONTEXT]\n## Intención: Listar contenido de un directorio\n- Bash/Zsh: `ls -la <ruta_directorio>`\n- Fish: `ls -la <ruta_directorio>`\n- PowerShell: `Get-ChildItem -Force -Path <ruta_directorio>`\n- CMD: `dir <ruta_directorio>`\n[END CONTEXT]\nRULE: Your response must have this exact two-line format, without adding markdown or greetings:\nEXPLANATION: [Your one-line explanation]\nCOMMAND: [The command extracted from context]\n<|im_end|>\n<|im_start|>user\nhazlo\n<|im_end|>\n<|im_start|>assistant\n";
        let generated = synthesize_response("qwen2.5-0.5b-instruct-q4_k_m", prompt);
        let lines: Vec<&str> = generated.lines().collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("EXPLANATION: "));
        assert!(lines[1].starts_with("COMMAND: "));
        assert_eq!(
            lines[1],
            "COMMAND: Get-ChildItem -Force -Path <ruta_directorio>"
        );
    }

    #[test]
    fn strict_how_to_prompt_produces_two_line_contract_spanish_locale() {
        // Verify legacy Spanish locale prompt still routes through the how-to
        // contract branch and produces the required two-line output.
        let prompt = "<|im_start|>system\nEres un asistente de terminal estricto. Tu único propósito es extraer el comando exacto para el subsystem: powershell, basándote EXCLUSIVAMENTE en el siguiente contexto.\n[CONTEXT]\n## Intención: Listar contenido de un directorio\n- Bash/Zsh: `ls -la <ruta_directorio>`\n- Fish: `ls -la <ruta_directorio>`\n- PowerShell: `Get-ChildItem -Force -Path <ruta_directorio>`\n- CMD: `dir <ruta_directorio>`\n[END CONTEXT]\nREGLA: Tu respuesta debe tener este formato exacto de dos líneas, sin añadir markdown ni saludos:\nEXPLANATION: [Tu explicación de una línea]\nCOMMAND: [El comando extraído del contexto]\n<|im_end|>\n<|im_start|>user\nhazlo\n<|im_end|>\n<|im_start|>assistant\n";
        let generated = synthesize_response("qwen2.5-0.5b-instruct-q4_k_m", prompt);
        let lines: Vec<&str> = generated.lines().collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("EXPLANATION: "));
        assert!(lines[1].starts_with("COMMAND: "));
    }

    #[test]
    fn extract_rag_context_supports_neutral_context_markers() {
        let prompt = "<|im_start|>system\n[CONTEXT]\nchunk one\n[END CONTEXT]\n<|im_end|>\n<|im_start|>user\nx\n<|im_end|>";
        let context = extract_rag_context(prompt);
        assert_eq!(context.as_deref(), Some("chunk one"));
    }

    #[test]
    fn extract_rag_context_supports_legacy_spanish_markers() {
        let prompt = "<|im_start|>system\n[CONTEXTO]\nchunk one\n[FIN CONTEXTO]\n<|im_end|>\n<|im_start|>user\nx\n<|im_end|>";
        let context = extract_rag_context(prompt);
        assert_eq!(context.as_deref(), Some("chunk one"));
    }

    fn unique_temp_file(name: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        std::env::temp_dir().join(format!("geli_shell_{millis}_{name}"))
    }
}
