use crate::shell::reporter::Reporter;
use crate::shell::translator::pipeline::context::TranslationContext;
use thiserror::Error;

// ══════════════════════════════════════════════════════════════
// PipelineError
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Error, PartialEq)]
pub enum PipelineError {
    /// El pipeline no puede continuar — error irrecuperable
    #[error("fatal pipeline error in step '{step}': {message}")]
    Fatal { step: &'static str, message: String },

    /// El step falló pero el pipeline puede continuar degradado
    #[error("degraded pipeline in step '{step}': {message}")]
    Degraded { step: &'static str, message: String },
}

impl PipelineError {
    pub fn fatal(step: &'static str, message: impl Into<String>) -> Self {
        Self::Fatal { step, message: message.into() }
    }

    pub fn degraded(step: &'static str, message: impl Into<String>) -> Self {
        Self::Degraded { step, message: message.into() }
    }

    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal { .. })
    }
}

// ══════════════════════════════════════════════════════════════
// StepResult
// ══════════════════════════════════════════════════════════════

#[derive(Debug, PartialEq)]
pub enum StepResult {
    /// Continúa al siguiente step
    Continue,

    /// Cortocircuita — output final ya está listo
    Done(String),
}

// ══════════════════════════════════════════════════════════════
// TranslationStep trait — contrato Open/Closed
// ══════════════════════════════════════════════════════════════

/// Contrato de un step del pipeline de traducción.
///
/// # Open/Closed
/// Para añadir nueva lógica de traducción:
/// - Implementa este trait en un nuevo struct
/// - Insértalo en `TranslationPipeline::default_steps()`
/// - No toques ningún step existente
///
/// # Responsabilidad única
/// Cada step hace exactamente una cosa:
/// - `NodeDecomposer`    → ASTNode → Vec<CommandFragment>
/// - `CommandResolver`   → name → CommandDef lookup
/// - `FlagResolver`      → --canonical → nativo
/// - `VariableExpander`  → $VAR → sintaxis subsistema
/// - `SubsystemMapper`   → ResolvedCommand → String
pub trait TranslationStep: Send + Sync {
    /// Nombre del step — usado en snapshots y mensajes de error
    fn name(&self) -> &'static str;

    /// Ejecuta el step sobre el contexto mutable.
    ///
    /// # Contract
    /// - Llama a `ctx.snapshot(self.name())` al final si tuvo éxito
    /// - Devuelve `StepResult::Continue` si el pipeline debe seguir
    /// - Devuelve `StepResult::Done(s)` para cortocircuitar
    /// - Devuelve `PipelineError::Fatal` si no puede continuar
    /// - Devuelve `PipelineError::Degraded` si puede continuar parcialmente
    fn process(
        &self,
        ctx:      &mut TranslationContext,
        reporter: &dyn Reporter,
    ) -> Result<StepResult, PipelineError>;
}
