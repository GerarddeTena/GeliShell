pub mod params;
pub mod qwen;
pub mod rag;
pub mod suggest;

use crate::shell::config::{AssistantConfig, ShellConfig};
use params::AssistantParameter;
use tokio::sync::mpsc;

#[derive(Debug, thiserror::Error)]
pub enum AssistantError {
    #[error("{0}")]
    Model(#[from] qwen::QwenError),

    #[error("{0}")]
    Rag(#[from] rag::RagError),
}

pub struct AssistantRuntime {
    settings: AssistantConfig,
    qwen: qwen::QwenRuntime,
    rag: rag::RagEngine,
}

impl AssistantRuntime {
    pub fn new(config: &ShellConfig) -> Self {
        Self {
            settings: config.assistant.clone(),
            qwen: qwen::QwenRuntime::new(ShellConfig::assistant_models_dir()),
            rag: rag::RagEngine::new(ShellConfig::assistant_models_dir()),
        }
    }

    pub fn refresh_config(&mut self, config: &ShellConfig) {
        self.settings = config.assistant.clone();
    }

    pub fn sweep_idle_resources(&mut self) {
        let _ = self
            .qwen
            .maybe_unload_idle(self.settings.auto_unload_after_secs);
    }

    pub async fn ensure_model_ready(
        &mut self,
        progress: mpsc::UnboundedSender<qwen::BootstrapEvent>,
    ) -> Result<qwen::ModelArtifact, AssistantError> {
        self.qwen
            .ensure_ready_and_load(self.settings.model_variant, progress)
            .await
            .map_err(AssistantError::from)
    }

    pub async fn run_parameter(
        &mut self,
        parameter: AssistantParameter,
        filter: &str,
    ) -> Result<suggest::AssistantSuggestion, AssistantError> {
        let user_action = suggest::build_user_action(parameter, filter);
        let retrieval_query = suggest::build_retrieval_query(parameter, filter);
        let rag_limit = 3usize;
        let rag_context = self
            .rag
            .retrieve_context(&retrieval_query, rag_limit)
            .await?;
        let llm_prompt = suggest::build_chatml_prompt(&user_action, &rag_context);
        let generated = self.qwen.generate(llm_prompt).await?;
        let suggestion = suggest::build_suggestion(generated);
        self.rag.clear_cache().await;
        Ok(suggestion)
    }

    pub async fn release_resources(&mut self) {
        self.qwen.unload();
        self.rag.clear_cache().await;
    }
}
