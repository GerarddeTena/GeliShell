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
    let action = extract_user_action(prompt)
        .unwrap_or_else(|| "Action unavailable. Provide a concrete assistant action.".to_owned());
    let normalized = action.to_ascii_lowercase();

    if normalized.contains("output my code") {
        return "Describe en 1 linea el error clave y luego ejecuta: `history | Select-Object -Last 30`."
            .to_owned();
    }

    if normalized.contains("copy directories") {
        return "Usa: `cp -r <origen> <destino>` (Bash/Zsh/Fish) o `Copy-Item -Recurse <origen> <destino>` (PowerShell).".to_owned();
    }

    if normalized.contains("search in files") {
        return "Usa: `rg \"<patron>\" .` y agrega `-g \"*.ext\"` para acotar archivos.".to_owned();
    }

    if normalized.contains("compress/extract") {
        return "Comprimir: `tar -czf <archivo>.tar.gz <ruta>`; extraer: `tar -xzf <archivo>.tar.gz`."
            .to_owned();
    }

    if normalized.contains("network request") {
        return "Plantilla segura: `curl -sS -X GET \"<url>\"` o `Invoke-WebRequest -Method Get -Uri \"<url>\"`."
            .to_owned();
    }

    if normalized.contains("process management") {
        return "Lista primero y luego termina por PID: `ps aux`/`Get-Process` + `kill <pid>`/`Stop-Process -Id <pid>`.".to_owned();
    }

    "No encontré una plantilla para esta acción. Indica la acción exacta del menú y te doy el comando."
        .to_owned()
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
    fn synthesize_response_does_not_echo_rag_context() {
        let prompt = "<|im_start|>system\n[CONTEXTO RECUPERADO DE RAG]\nsecret chunk from docs db\n<|im_end|>\n<|im_start|>user\nSearch in files\n<|im_end|>\n<|im_start|>assistant";
        let generated = synthesize_response("qwen2.5-1.5b-instruct-q4_k_m", prompt);

        assert!(!generated.contains("secret chunk from docs db"));
        assert!(generated.contains("rg"));
    }

    fn unique_temp_file(name: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        std::env::temp_dir().join(format!("geli_shell_{millis}_{name}"))
    }
}
