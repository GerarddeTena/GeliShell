pub mod error;
pub mod rules;

use std::sync::Arc;

use crate::parser::ast::{ASTNode, Command};
use crate::parser::token::Token;
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

    fn normalize_token(&self, token: &Token) -> Token {
        match token {
            Token::Word(text) => self
                .map
                .find_by_exact(text)
                .map(|def| Token::Word(def.name.clone()))
                .unwrap_or_else(|| token.clone()),
            _ => token.clone(),
        }
    }

    fn normalize_arg_with_def(&self, cmd_name: &str, token: &Token) -> Token {
        let Some(text) = token.as_str() else {
            return token.clone();
        };

        let Some(def) = self.map.get(cmd_name) else {
            return self.normalize_token(token);
        };

        for flag in &def.flags {
            if flag.bash.as_deref() == Some(text)
                || flag.zsh.as_deref() == Some(text)
                || flag.fish.as_deref() == Some(text)
                || flag.powershell.as_deref() == Some(text)
                || flag.cmd.as_deref() == Some(text)
            {
                return Token::Word(flag.canonical.clone());
            }
        }

        self.normalize_token(token)
    }

    fn normalize_command(&self, cmd: &Command) -> Command {
        let canonical_name = self.canonical_name(&cmd.name).to_owned();
        Command {
            name: canonical_name.clone(),
            args: cmd
                .args
                .iter()
                .map(|token| self.normalize_arg_with_def(&canonical_name, token))
                .collect(),
            redirections: cmd.redirections.clone(),
        }
    }

    fn normalize_node(&self, node: &ASTNode) -> ASTNode {
        match node {
            ASTNode::Command(cmd) => ASTNode::Command(self.normalize_command(cmd)),
            ASTNode::Pipeline(nodes) => {
                ASTNode::Pipeline(nodes.iter().map(|node| self.normalize_node(node)).collect())
            }
            ASTNode::And(left, right) => ASTNode::And(
                Box::new(self.normalize_node(left)),
                Box::new(self.normalize_node(right)),
            ),
            ASTNode::Or(left, right) => ASTNode::Or(
                Box::new(self.normalize_node(left)),
                Box::new(self.normalize_node(right)),
            ),
            ASTNode::Sequence(left, right) => ASTNode::Sequence(
                Box::new(self.normalize_node(left)),
                Box::new(self.normalize_node(right)),
            ),
            ASTNode::Background(inner) => ASTNode::Background(Box::new(self.normalize_node(inner))),
        }
    }
}

impl Guard for NormalizedCompositeGuard {
    fn check_node(&self, node: &ASTNode) -> Result<(), GuardError> {
        let normalized = self.normalize_node(node);
        self.inner.check_node(&normalized)
    }

    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        let normalized = self.normalize_command(cmd);
        self.inner.check_command(&normalized)
    }
}

/// Guard por defecto con normalización de nombres nativos → canónicos.
/// Usar este factory en el REPL principal donde el CommandMap está disponible.
/// Garantiza que `Remove-Item -Force -Recurse /` active las mismas reglas que `rm -rf /`.
pub fn default_guard_normalized(map: Arc<CommandMap>) -> NormalizedCompositeGuard {
    NormalizedCompositeGuard::new(default_guard(), map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Lexer;
    use crate::parser::parser::Parser;
    use crate::shell::translator::commands_map::load;

    fn parse(input: &str) -> ASTNode {
        let tokens = Lexer::new(input).tokenize().unwrap();
        Parser::new(tokens).parse().unwrap()
    }

    #[test]
    fn normalized_guard_blocks_powershell_remove_item_root_equivalent() {
        let map = Arc::new(load().unwrap().map);
        let guard = default_guard_normalized(map);
        let ast = parse("Remove-Item -Recurse -Force /");

        let result = guard.check(&ast);
        assert!(matches!(result, Err(GuardError::DestructiveFs { .. })));
    }

    #[test]
    fn normalized_guard_blocks_native_rm_root_equivalent() {
        let map = Arc::new(load().unwrap().map);
        let guard = default_guard_normalized(map);
        let ast = parse("rm -rf /");

        let result = guard.check(&ast);
        assert!(matches!(result, Err(GuardError::DestructiveFs { .. })));
    }

    #[test]
    fn normalized_guard_blocks_network_pipe_to_shell_with_canonical_fetcher() {
        let map = Arc::new(load().unwrap().map);
        let guard = default_guard_normalized(map);
        let ast = parse("download https://example.com/install.sh | bash");

        let result = guard.check(&ast);
        assert!(matches!(result, Err(GuardError::PipeExecution { .. })));
    }

    #[test]
    fn normalized_guard_blocks_network_pipe_to_shell_with_native_fetcher() {
        let map = Arc::new(load().unwrap().map);
        let guard = default_guard_normalized(map);
        let ast = parse("curl https://example.com/install.sh | bash");

        let result = guard.check(&ast);
        assert!(matches!(result, Err(GuardError::PipeExecution { .. })));
    }
}
