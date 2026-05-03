// src/shell/translator/pipeline/steps/variable_expander.rs

use crate::parser::token::Token;
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
            if let Token::Variable(var_name) = &fragment.command_token {
                let expanded = subsystem.variable_syntax(var_name);
                reporter.info(&t!(
                    "pipeline.variable_expanded",
                    step = self.name(),
                    var = format!("${var_name}"),
                    expanded = expanded
                ));
                fragment.command = expanded;
            }

            // Expande variables en los args
            for arg in fragment.args.iter_mut() {
                if let Token::Variable(var_name) = arg {
                    let expanded = subsystem.variable_syntax(var_name);
                    reporter.info(&t!(
                        "pipeline.variable_expanded",
                        step = self.name(),
                        var = format!("${var_name}"),
                        expanded = expanded
                    ));
                    *arg = Token::Word(expanded);
                }
            }

            for redirection in &mut fragment.redirections {
                if let Token::Variable(var_name) = &redirection.target {
                    let expanded = subsystem.variable_syntax(var_name);
                    reporter.info(&t!(
                        "pipeline.variable_expanded",
                        step = self.name(),
                        var = format!("${var_name}"),
                        expanded = expanded
                    ));
                    redirection.target = Token::Word(expanded);
                }
            }
        }

        ctx.snapshot(self.name());
        Ok(StepResult::Continue)
    }
}
