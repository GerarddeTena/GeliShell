mod error;
mod destructive_fs;

use crate::parser::ast::{ASTNode, Command};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum GuardError {
    #[error("comando prohibido: '{name} {}'", .args.join(" "))]
    BlacklistedCommand { name: String, args: Vec<String> },
    #[error("argumento prohibido '{arg}' en '{command}'")]
    ForbiddenArgument { command: String, arg: String },
}

pub trait Guard {
    /// Punto de entrada — recorre el AST completo
    fn check(&self, node: &ASTNode) -> Result<(), GuardError> {
        self.check_node(node)
    }

    #[doc(hidden)]
    fn check_node(&self, node: &ASTNode) -> Result<(), GuardError> {
        match node {
            ASTNode::Command(cmd) => self.check_command(cmd),

            // Recorre recursivamente cada rama del árbol
            ASTNode::Pipeline(commands) => {
                commands.iter().try_for_each(|n| self.check_node(n))
            }
            ASTNode::And(left, right)
            | ASTNode::Or(left, right)
            | ASTNode::Sequence(left, right) => {
                self.check_node(left)?;
                self.check_node(right)
            }
            ASTNode::Background(inner) => self.check_node(inner),
        }
    }

    /// Implementar en cada guard concreto
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError>;
}
pub(crate) struct BlacklistGuard {
    /// Cada entrada es (nombre, args_prohibidos)
    /// ("rm", vec!["-rf"]) bloquea "rm -rf" pero no "rm archivo.txt"
    rules: Vec<(String, Vec<String>)>,
}

impl BlacklistGuard {
    pub(crate) fn new(rules: Vec<(&str, Vec<&str>)>) -> Self {
        BlacklistGuard {
            rules: rules
                .into_iter()
                .map(|(cmd, args)| {
                    (cmd.to_string(), args.into_iter().map(String::from).collect())
                })
                .collect(),
        }
    }
}

impl Guard for BlacklistGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        let cmd_args: Vec<String> = cmd
            .args
            .iter()
            .filter_map(|t| t.as_str().map(str::to_owned))
            .collect();

        for (blocked_name, blocked_args) in &self.rules {
            if cmd.name != *blocked_name {
                continue;
            }
            // Todos los args prohibidos deben estar presentes
            let all_match = blocked_args.iter().all(|a| cmd_args.contains(a));
            if all_match {
                return Err(GuardError::BlacklistedCommand {
                    name: cmd.name.clone(),
                    args: cmd_args,
                });
            }
        }
        Ok(())
    }
}
pub(crate) struct ArgPatternGuard {
    /// Comandos con sus patrones prohibidos en args
    /// ("curl", vec!["--insecure"]) bloquea curl --insecure
    rules: Vec<(String, Vec<String>)>,
}

impl ArgPatternGuard {
    pub(crate) fn new(rules: Vec<(&str, Vec<&str>)>) -> Self {
        ArgPatternGuard {
            rules: rules
                .into_iter()
                .map(|(cmd, patterns)| {
                    (cmd.to_string(), patterns.into_iter().map(String::from).collect())
                })
                .collect(),
        }
    }
}

impl Guard for ArgPatternGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        let cmd_args: Vec<String> = cmd
            .args
            .iter()
            .filter_map(|t| t.as_str().map(str::to_owned))
            .collect();

        for (guarded_cmd, forbidden_patterns) in &self.rules {
            if cmd.name != *guarded_cmd {
                continue;
            }
            for pattern in forbidden_patterns {
                if cmd_args.iter().any(|a: &String| a.contains(pattern.as_str())) {
                    return Err(GuardError::ForbiddenArgument {
                        command: cmd.name.clone(),
                        arg: pattern.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

pub struct CompositeGuard {
    guards: Vec<Box<dyn Guard>>,
}

impl CompositeGuard {
    pub fn new(guards: Vec<Box<dyn Guard>>) -> Self {
        CompositeGuard { guards }
    }
}

pub fn default_guard(
    blacklist: Vec<(&str, Vec<&str>)>,
    patterns: Vec<(&str, Vec<&str>)>,
) -> CompositeGuard {
    CompositeGuard::new(vec![
        Box::new(BlacklistGuard::new(blacklist)),
        Box::new(ArgPatternGuard::new(patterns)),
    ])
}

impl Guard for CompositeGuard {
    // check_node ya está en el trait — solo necesitamos check_command
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        self.guards.iter().try_for_each(|g| g.check_command(cmd))
    }
}
