use crate::shell::reporter::Reporter;
use crate::shell::translator::pipeline::context::TranslationContext;
use crate::shell::translator::pipeline::step::{PipelineError, StepResult, TranslationStep};
use crate::shell::translator::resolver::Resolve;
use crate::t;
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
    fn name(&self) -> &'static str {
        "SubsystemMapper"
    }

    fn process(
        &self,
        ctx: &mut TranslationContext,
        reporter: &dyn Reporter,
    ) -> Result<StepResult, PipelineError> {
        let subsystem = ctx.subsystem;

        for fragment in ctx.fragments.iter_mut() {
            let Some(def) = &fragment.command_def else {
                // Pass-through — sin CommandDef no hay mapping
                reporter.info(&t!("pipeline.mapped_passthrough", step = self.name(), command = fragment.command));
                continue;
            };

            match self.resolver.resolve(def, subsystem, reporter) {
                Ok(resolved) => {
                    reporter.info(&t!("pipeline.mapped",
                        step = self.name(),
                        command = fragment.command,
                        resolved = resolved.preferred,
                        count = resolved.alternatives.len()
                    ));
                    fragment.command = resolved.preferred.clone();
                    fragment.resolved = Some(resolved);
                }
                Err(e) => {
                    // Degraded — usa el comando original como fallback
                    reporter.warn(&t!("pipeline.resolver_error",
                        step = self.name(),
                        command = fragment.command,
                        error = e
                    ));
                }
            }
        }

        ctx.snapshot(self.name());
        Ok(StepResult::Continue)
    }
}
