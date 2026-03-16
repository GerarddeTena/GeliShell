use crate::shell::reporter::Reporter;
use crate::shell::translator::pipeline::context::TranslationContext;
use crate::shell::translator::pipeline::step::{
    PipelineError, StepResult, TranslationStep,
};
use crate::shell::translator::resolver::Resolve;
use std::sync::Arc;

pub struct SubsystemMapper {
    resolver: Arc<dyn Resolve>,
}

impl SubsystemMapper {
    pub fn new(resolver: Arc<dyn Resolve>) -> Self {
        Self { resolver }
    }
}

impl TranslationStep for SubsystemMapper {
    fn name(&self) -> &'static str { "SubsystemMapper" }

    fn process(
        &self,
        ctx:      &mut TranslationContext,
        reporter: &dyn Reporter,
    ) -> Result<StepResult, PipelineError> {
        let subsystem = ctx.subsystem;

        for fragment in ctx.fragments.iter_mut() {
            let Some(def) = &fragment.command_def else {
                // Pass-through — sin CommandDef no hay mapping
                reporter.info(&format!(
                    "{}: '{}' → pass-through (no canonical def)",
                    self.name(), fragment.command
                ));
                continue;
            };

            match self.resolver.resolve(def, subsystem, reporter) {
                Ok(resolved) => {
                    reporter.info(&format!(
                        "{}: '{}' → '{}' [{} alternatives]",
                        self.name(),
                        fragment.command,
                        resolved.preferred,
                        resolved.alternatives.len()
                    ));
                    fragment.command  = resolved.preferred.clone();
                    fragment.resolved = Some(resolved);
                }
                Err(e) => {
                    // Degraded — usa el comando original como fallback
                    reporter.warn(&format!(
                        "{}: resolver error for '{}': {e} — using original",
                        self.name(), fragment.command
                    ));
                }
            }
        }

        ctx.snapshot(self.name());
        Ok(StepResult::Continue)
    }
}