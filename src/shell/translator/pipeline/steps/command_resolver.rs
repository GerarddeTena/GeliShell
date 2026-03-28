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
                    // Reverse lookup: el usuario escribió un comando nativo
                    // (p.ej. `ls` en PowerShell o `Get-ChildItem` en Bash).
                    // Si hay una correspondencia canónica unívoca, normalizamos
                    // el fragmento al nombre canónico para que el pipeline
                    // lo traduzca al subsistema activo.
                    if let Some(def) = ctx.map.find_by_exact(&fragment.command) {
                        reporter.info(&format!(
                            "{}: '{}' → reverse lookup → canonical '{}'",
                            self.name(),
                            fragment.command,
                            def.name,
                        ));
                        fragment.command = def.name.clone();
                        fragment.command_def = Some(def.clone());
                    } else {
                        // Pass-through — comando nativo sin equivalente canónico
                        reporter.info(&format!(
                            "{}: '{}' → no canonical match, pass-through",
                            self.name(),
                            fragment.command
                        ));
                    }
                }
            }
        }

        ctx.snapshot(self.name());
        Ok(StepResult::Continue)
    }
}
