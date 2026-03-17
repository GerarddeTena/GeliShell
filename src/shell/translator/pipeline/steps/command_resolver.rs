use crate::shell::reporter::Reporter;
use crate::shell::translator::pipeline::context::TranslationContext;
use crate::shell::translator::pipeline::step::{PipelineError, StepResult, TranslationStep};

pub struct CommandResolver;

impl CommandResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommandResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationStep for CommandResolver {
    fn name(&self) -> &'static str {
        "CommandResolver"
    }

    fn process(
        &self,
        ctx: &mut TranslationContext,
        reporter: &dyn Reporter,
    ) -> Result<StepResult, PipelineError> {
        for fragment in ctx.fragments.iter_mut() {
            match ctx.map.get(&fragment.command) {
                Some(def) => {
                    reporter.info(&format!(
                        "{}: '{}' → canonical match found",
                        self.name(),
                        fragment.command
                    ));
                    fragment.command_def = Some(def.clone());
                }
                None => {
                    // Pass-through — comando nativo no canónico
                    // No es un error — el usuario puede escribir
                    // comandos nativos directamente
                    reporter.info(&format!(
                        "{}: '{}' → no canonical match, pass-through",
                        self.name(),
                        fragment.command
                    ));
                }
            }
        }

        ctx.snapshot(self.name());
        Ok(StepResult::Continue)
    }
}
