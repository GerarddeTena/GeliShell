use crate::parser::ast::ASTNode;
use crate::parser::token::Token;
use crate::shell::reporter::Reporter;
use crate::shell::translator::pipeline::context::{
    CommandFragment, FragmentOperator, TranslationContext,
};
use crate::shell::translator::pipeline::step::{PipelineError, StepResult, TranslationStep};
use crate::t;

pub struct NodeDecomposer;

impl NodeDecomposer {
    pub fn new() -> Self {
        Self
    }

    /// Descompone recursivamente un ASTNode en Vec<CommandFragment>
    /// Esta es la única función del sistema que hace match sobre ASTNode
    fn decompose(&self, node: &ASTNode, out: &mut Vec<CommandFragment>, reporter: &dyn Reporter) {
        match node {
            ASTNode::Command(cmd) => {
                let args = cmd
                    .args
                    .iter()
                    .filter_map(|t| match t {
                        Token::Word(s) | Token::Quoted(s) | Token::Variable(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();

                out.push(CommandFragment::new(cmd.name.clone(), args));
            }

            ASTNode::Pipeline(nodes) => {
                let last_idx = nodes.len().saturating_sub(1);
                for (i, n) in nodes.iter().enumerate() {
                    self.decompose(n, out, reporter);
                    // Todos los fragments de este nodo excepto el último llevan Pipe
                    if i < last_idx {
                        if let Some(last) = out.last_mut() {
                            last.operator = Some(FragmentOperator::Pipe);
                        }
                    }
                }
            }

            ASTNode::And(left, right) => {
                self.decompose(left, out, reporter);
                if let Some(last) = out.last_mut() {
                    last.operator = Some(FragmentOperator::And);
                }
                self.decompose(right, out, reporter);
            }

            ASTNode::Or(left, right) => {
                self.decompose(left, out, reporter);
                if let Some(last) = out.last_mut() {
                    last.operator = Some(FragmentOperator::Or);
                }
                self.decompose(right, out, reporter);
            }

            ASTNode::Sequence(left, right) => {
                self.decompose(left, out, reporter);
                if let Some(last) = out.last_mut() {
                    last.operator = Some(FragmentOperator::Sequence);
                }
                self.decompose(right, out, reporter);
            }

            ASTNode::Background(inner) => {
                let before = out.len();
                self.decompose(inner, out, reporter);
                // Marca todos los fragments generados como background
                for f in out[before..].iter_mut() {
                    f.background = true;
                }
            }
        }
    }
}

impl Default for NodeDecomposer {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationStep for NodeDecomposer {
    fn name(&self) -> &'static str {
        "NodeDecomposer"
    }

    fn process(
        &self,
        ctx: &mut TranslationContext,
        reporter: &dyn Reporter,
    ) -> Result<StepResult, PipelineError> {
        if !ctx.fragments.is_empty() {
            return Err(PipelineError::fatal(
                self.name(),
                "NodeDecomposer must run first — fragments already present",
            ));
        }

        self.decompose(ctx.node, &mut ctx.fragments, reporter);

        if ctx.fragments.is_empty() {
            return Err(PipelineError::fatal(
                self.name(),
                "decomposition produced no fragments",
            ));
        }

        reporter.info(&t!("pipeline.decomposed", step = self.name(), count = ctx.fragments.len()));

        ctx.snapshot(self.name());
        Ok(StepResult::Continue)
    }
}
