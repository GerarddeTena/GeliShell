// src/shell/translator/pipeline/steps/variable_expander.rs

use crate::shell::reporter::Reporter;
use crate::shell::translator::pipeline::context::TranslationContext;
use crate::shell::translator::pipeline::step::{PipelineError, StepResult, TranslationStep};
use crate::t;

pub struct VariableExpander;

impl VariableExpander {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VariableExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationStep for VariableExpander {
    fn name(&self) -> &'static str {
        "VariableExpander"
    }

    fn process(
        &self,
        ctx: &mut TranslationContext,
        reporter: &dyn Reporter,
    ) -> Result<StepResult, PipelineError> {
        let subsystem = ctx.subsystem;

        for fragment in ctx.fragments.iter_mut() {
            // Expande variables en el nombre del comando
            if fragment.command.starts_with('$') {
                let var_name = fragment.command.trim_start_matches('$');
                fragment.command = subsystem.variable_syntax(var_name);
            }

            // Expande variables en los args
            for arg in fragment.args.iter_mut() {
                if arg.starts_with('$') {
                    let var_name = arg.trim_start_matches('$');
                    let expanded = subsystem.variable_syntax(var_name);
                    reporter.info(&t!(
                        "pipeline.variable_expanded",
                        step = self.name(),
                        var = arg,
                        expanded = expanded
                    ));
                    *arg = expanded;
                }
            }
        }

        ctx.snapshot(self.name());
        Ok(StepResult::Continue)
    }
}
