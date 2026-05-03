use crate::parser::ast::{ASTNode, Command};
use crate::shell::guard::Guard;
use crate::shell::guard::error::GuardError;
use crate::t;

const NETWORK_FETCHERS: &[&str] = &["curl", "wget", "fetch", "http", "http-get", "download"];
const SHELL_EXECUTORS: &[&str] = &[
    "bash",
    "sh",
    "zsh",
    "fish",
    "dash",
    "ksh",
        "geliShell",
        "geli",
        "run-background",
    ];

pub struct PipeExecutionGuard;

impl PipeExecutionGuard {
    pub fn new() -> Self {
        Self
    }

    fn is_network_fetcher(name: &str) -> bool {
        NETWORK_FETCHERS.contains(&name)
    }

    fn is_shell_executor(name: &str) -> bool {
        SHELL_EXECUTORS.contains(&name)
    }
}

impl Default for PipeExecutionGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Guard for PipeExecutionGuard {
    // Sobreescribe check_node para analizar pipelines completos
    fn check_node(&self, node: &ASTNode) -> Result<(), GuardError> {
        if let ASTNode::Pipeline(commands) = node {
            // Detecta: fetcher | executor
            // Itera pares adyacentes en el pipeline
            for window in commands.windows(2) {
                let left = &window[0];
                let right = &window[1];

                let left_is_fetcher = if let ASTNode::Command(cmd) = left {
                    Self::is_network_fetcher(&cmd.name)
                } else {
                    false
                };

                let right_is_executor = if let ASTNode::Command(cmd) = right {
                    Self::is_shell_executor(&cmd.name)
                } else {
                    false
                };

                if left_is_fetcher && right_is_executor {
                    return Err(GuardError::PipeExecution {
                        reason: t!("guard.pipe_execution.network_pipe_blocked"),
                    });
                }
            }
        }
        // Recorre el resto del árbol
        match node {
            ASTNode::Command(cmd) => self.check_command(cmd),
            ASTNode::Pipeline(_) => Ok(()), // ya procesado arriba
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
