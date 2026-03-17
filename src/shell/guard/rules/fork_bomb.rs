use crate::parser::ast::{ASTNode, Command};
use crate::shell::guard::Guard;
use crate::shell::guard::error::GuardError;

/// Detecta patrones de fork bomb en el AST.
///
/// El patrón clásico: :(){ :|:& };:
/// En el AST aparece como:
/// Background(Pipeline([Command("X"), Command("X")]))
/// donde X se llama a sí mismo
pub struct ForkBombGuard;

impl ForkBombGuard {
    pub fn new() -> Self {
        Self
    }

    /// Comprueba si un nodo es un pipeline donde el mismo
    /// comando aparece en ambos lados y se ejecuta en background
    fn is_fork_bomb_pattern(node: &ASTNode) -> bool {
        if let ASTNode::Background(inner) = node {
            if let ASTNode::Pipeline(cmds) = inner.as_ref() {
                if cmds.len() >= 2 {
                    let names: Vec<&str> = cmds
                        .iter()
                        .filter_map(|n| {
                            if let ASTNode::Command(cmd) = n {
                                Some(cmd.name.as_str())
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Todos los comandos del pipeline son iguales
                    // y hay al menos 2 — patrón de fork bomb
                    if names.len() >= 2 {
                        let first = names[0];
                        if names.iter().all(|&n| n == first) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl Guard for ForkBombGuard {
    fn check_node(&self, node: &ASTNode) -> Result<(), GuardError> {
        if Self::is_fork_bomb_pattern(node) {
            return Err(GuardError::ForkBomb);
        }
        // Recorre recursivamente
        match node {
            ASTNode::Command(cmd) => self.check_command(cmd),
            ASTNode::Pipeline(cmds) => cmds.iter().try_for_each(|n| self.check_node(n)),
            ASTNode::And(l, r) | ASTNode::Or(l, r) | ASTNode::Sequence(l, r) => {
                self.check_node(l)?;
                self.check_node(r)
            }
            ASTNode::Background(inner) => self.check_node(inner),
        }
    }

    fn check_command(&self, _cmd: &Command) -> Result<(), GuardError> {
        Ok(())
    }
}
