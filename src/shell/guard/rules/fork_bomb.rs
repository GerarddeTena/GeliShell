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
        if let ASTNode::Background(inner) = node
            && let ASTNode::Pipeline(cmds) = inner.as_ref()
            && cmds.len() >= 2
        {
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
        false
    }
}

impl Default for ForkBombGuard {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::Command;

    fn cmd(name: &str) -> ASTNode {
        ASTNode::Command(Command {
            name: name.to_owned(),
            args: vec![],
            redirections: vec![],
        })
    }

    fn pipeline(cmds: Vec<ASTNode>) -> ASTNode {
        ASTNode::Pipeline(cmds)
    }

    fn background(node: ASTNode) -> ASTNode {
        ASTNode::Background(Box::new(node))
    }

    #[test]
    fn classic_fork_bomb_detected() {
        // :(){ :|:& };: → Background(Pipeline([cmd(":"), cmd(":")]))
        let node = background(pipeline(vec![cmd(":"), cmd(":")]));
        assert_eq!(
            ForkBombGuard::new().check_node(&node),
            Err(GuardError::ForkBomb)
        );
    }

    #[test]
    fn three_stage_fork_bomb_detected() {
        // x|x|x & — three identical commands in background pipeline
        let node = background(pipeline(vec![cmd("x"), cmd("x"), cmd("x")]));
        assert_eq!(
            ForkBombGuard::new().check_node(&node),
            Err(GuardError::ForkBomb)
        );
    }

    #[test]
    fn mixed_commands_in_pipeline_not_blocked() {
        // a|b & — different commands, not a fork bomb
        let node = background(pipeline(vec![cmd("a"), cmd("b")]));
        assert!(ForkBombGuard::new().check_node(&node).is_ok());
    }

    #[test]
    fn pipeline_without_background_not_blocked() {
        // a|a (no &) — self-pipe but not backgrounded
        let node = pipeline(vec![cmd("a"), cmd("a")]);
        assert!(ForkBombGuard::new().check_node(&node).is_ok());
    }

    #[test]
    fn single_command_not_blocked() {
        assert!(ForkBombGuard::new().check_node(&cmd("echo")).is_ok());
    }

    #[test]
    fn fork_bomb_nested_in_sequence_detected() {
        // echo; :(){ :|:& };:
        let bomb = background(pipeline(vec![cmd(":"), cmd(":")]));
        let node = ASTNode::Sequence(Box::new(cmd("echo")), Box::new(bomb));
        assert_eq!(
            ForkBombGuard::new().check_node(&node),
            Err(GuardError::ForkBomb)
        );
    }

    #[test]
    fn fork_bomb_nested_in_and_detected() {
        let bomb = background(pipeline(vec![cmd("x"), cmd("x")]));
        let node = ASTNode::And(Box::new(cmd("true")), Box::new(bomb));
        assert_eq!(
            ForkBombGuard::new().check_node(&node),
            Err(GuardError::ForkBomb)
        );
    }

    #[test]
    fn fork_bomb_nested_in_or_detected() {
        let bomb = background(pipeline(vec![cmd("x"), cmd("x")]));
        let node = ASTNode::Or(Box::new(cmd("false")), Box::new(bomb));
        assert_eq!(
            ForkBombGuard::new().check_node(&node),
            Err(GuardError::ForkBomb)
        );
    }

    #[test]
    fn background_single_command_not_blocked() {
        // echo & — background but not a pipeline fork bomb
        let node = background(cmd("echo"));
        assert!(ForkBombGuard::new().check_node(&node).is_ok());
    }
}
