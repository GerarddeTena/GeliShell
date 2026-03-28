use crate::shell::reporter::Reporter;
use crate::shell::translator::pipeline::context::TranslationContext;
use crate::shell::translator::pipeline::step::{PipelineError, StepResult, TranslationStep};
use crate::t;

pub struct FlagResolver;

impl FlagResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlagResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationStep for FlagResolver {
    fn name(&self) -> &'static str {
        "FlagResolver"
    }

    fn process(
        &self,
        ctx: &mut TranslationContext,
        reporter: &dyn Reporter,
    ) -> Result<StepResult, PipelineError> {
        let subsystem = ctx.subsystem;

        for fragment in ctx.fragments.iter_mut() {
            // Solo resuelve flags si tenemos un CommandDef
            // para este fragment — si no, pass-through
            let Some(def) = &fragment.command_def else {
                continue;
            };

            let mut resolved_args: Vec<String> = Vec::with_capacity(fragment.args.len());

            for arg in &fragment.args {
                if arg.starts_with("--") {
                    // Busca el flag canónico en el CommandDef
                    match def.flags.iter().find(|f| f.canonical == *arg) {
                        Some(flag_def) => {
                            match flag_def.get_by_name(subsystem.as_str()) {
                                Some(translated) => {
                                    reporter.info(&t!("pipeline.flag_resolved",
                                        step = self.name(),
                                        flag = arg,
                                        translated = translated
                                    ));
                                    resolved_args.push(translated.to_owned());
                                }
                                None => {
                                    // Flag no soportado en este subsistema
                                    // Degraded — continúa sin este flag
                                    reporter.warn(&t!("pipeline.flag_not_supported",
                                        step = self.name(),
                                        flag = arg,
                                        subsystem = subsystem.as_str()
                                    ));
                                }
                            }
                        }
                        None => {
                            // Flag no registrado — pass-through
                            resolved_args.push(arg.clone());
                        }
                    }
                } else {
                    // No es un flag canónico — pass-through
                    resolved_args.push(arg.clone());
                }
            }

            fragment.args = resolved_args;
        }

        ctx.snapshot(self.name());
        Ok(StepResult::Continue)
    }
}
