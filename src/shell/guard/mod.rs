pub mod error;
pub mod rules;

use crate::parser::ast::{ASTNode, Command};
pub use error::GuardError;
use rules::{
    ChmodChownGuard, CriticalRedirectGuard, DdGuard,
    ForkBombGuard, MkfsGuard, PipeExecutionGuard, RmGuard,
};

// ══════════════════════════════════════════════════════════════
// trait Guard
// ══════════════════════════════════════════════════════════════

pub trait Guard: Send + Sync {
    fn check(&self, node: &ASTNode) -> Result<(), GuardError> {
        self.check_node(node)
    }

    #[doc(hidden)]
    fn check_node(&self, node: &ASTNode) -> Result<(), GuardError> {
        match node {
            ASTNode::Command(cmd) => self.check_command(cmd),
            ASTNode::Pipeline(commands) => {
                commands.iter().try_for_each(|n| self.check_node(n))
            }
            ASTNode::And(l, r)
            | ASTNode::Or(l, r)
            | ASTNode::Sequence(l, r) => {
                self.check_node(l)?;
                self.check_node(r)
            }
            ASTNode::Background(inner) => self.check_node(inner),
        }
    }

    fn check_command(&self, cmd: &Command) -> Result<(), GuardError>;
}

// ══════════════════════════════════════════════════════════════
// CompositeGuard
// ══════════════════════════════════════════════════════════════

pub struct CompositeGuard {
    guards: Vec<Box<dyn Guard>>,
}

impl CompositeGuard {
    pub fn new(guards: Vec<Box<dyn Guard>>) -> Self {
        Self { guards }
    }
}

impl Guard for CompositeGuard {
    fn check_node(&self, node: &ASTNode) -> Result<(), GuardError> {
        self.guards.iter().try_for_each(|g| g.check_node(node))
    }

    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        self.guards.iter().try_for_each(|g| g.check_command(cmd))
    }
}

// ══════════════════════════════════════════════════════════════
// Factory — guard por defecto con todas las reglas activas
// ══════════════════════════════════════════════════════════════

pub fn default_guard() -> CompositeGuard {
    CompositeGuard::new(vec![
        Box::new(RmGuard::new()),
        Box::new(ChmodChownGuard::new()),
        Box::new(DdGuard::new()),
        Box::new(MkfsGuard::new()),
        Box::new(CriticalRedirectGuard::new()),
        Box::new(PipeExecutionGuard::new()),
        Box::new(ForkBombGuard::new()),
    ])
}