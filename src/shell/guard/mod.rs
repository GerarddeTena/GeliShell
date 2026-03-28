pub mod error;
pub mod rules;

use std::sync::Arc;

use crate::parser::ast::{ASTNode, Command};
use crate::shell::translator::commands_map::CommandMap;
pub use error::GuardError;
use rules::{
    ChmodChownGuard, CriticalRedirectGuard, DdGuard, ForkBombGuard, MkfsGuard, PipeExecutionGuard,
    RmGuard,
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
            ASTNode::Pipeline(commands) => commands.iter().try_for_each(|n| self.check_node(n)),
            ASTNode::And(l, r) | ASTNode::Or(l, r) | ASTNode::Sequence(l, r) => {
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

// ══════════════════════════════════════════════════════════════
// NormalizedCompositeGuard — normaliza nombres nativos → canónicos
// antes de delegar al CompositeGuard interno.
//
// Problema que resuelve: el Guard opera sobre el AST crudo (pre-traducción).
// Sin normalización, `Remove-Item -Force -Recurse /` bypasa RmGuard porque
// `cmd.name == "Remove-Item"` ≠ `"rm"`. Con normalización, el reverse_index
// del CommandMap convierte el nombre nativo al canónico antes de la comprobación.
// ══════════════════════════════════════════════════════════════

pub struct NormalizedCompositeGuard {
    inner: CompositeGuard,
    map: Arc<CommandMap>,
}

impl NormalizedCompositeGuard {
    pub fn new(inner: CompositeGuard, map: Arc<CommandMap>) -> Self {
        Self { inner, map }
    }

    fn canonical_name<'a>(&'a self, name: &'a str) -> &'a str {
        self.map
            .find_by_exact(name)
            .map(|def| def.name.as_str())
            .unwrap_or(name)
    }
}

impl Guard for NormalizedCompositeGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        let canonical = self.canonical_name(&cmd.name);
        if canonical != cmd.name {
            // Construye un Command temporal con el nombre canónico manteniendo
            // los mismos args y redirections para que las reglas existentes funcionen
            let normalized = Command {
                name: canonical.to_owned(),
                args: cmd.args.clone(),
                redirections: cmd.redirections.clone(),
            };
            self.inner.check_command(&normalized)
        } else {
            self.inner.check_command(cmd)
        }
    }
}

/// Guard por defecto con normalización de nombres nativos → canónicos.
/// Usar este factory en el REPL principal donde el CommandMap está disponible.
/// Garantiza que `Remove-Item -Force -Recurse /` active las mismas reglas que `rm -rf /`.
pub fn default_guard_normalized(map: Arc<CommandMap>) -> NormalizedCompositeGuard {
    NormalizedCompositeGuard::new(default_guard(), map)
}
