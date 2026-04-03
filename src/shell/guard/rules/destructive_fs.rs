use crate::parser::ast::Command;
use crate::parser::token::Token;
use crate::shell::guard::Guard;
use crate::shell::guard::error::GuardError;
use crate::t;

/// Targets de raíz que nunca deben ser objetivo de operaciones destructivas
const ROOT_TARGETS: &[&str] = &[
    "/", "/*", "/etc", "/etc/*", "/boot", "/boot/*", "/usr", "/usr/*", "/bin", "/sbin", "/lib",
    "/lib64",
];

// ══════════════════════════════════════════════════════════════
// RmGuard — El Destructor de Mundos
// ══════════════════════════════════════════════════════════════

pub struct RmGuard;

impl RmGuard {
    pub fn new() -> Self {
        Self
    }

    fn has_recursive(args: &[String]) -> bool {
        args.iter().any(|a| {
            matches!(
                a.as_str(),
                "-r" | "-R" | "--recursive" | "-rf" | "-fr" | "-Rf" | "-fR" | "-rR" | "-Rr"
            )
        })
    }

    fn has_force(args: &[String]) -> bool {
        args.iter()
            .any(|a| matches!(a.as_str(), "-f" | "--force" | "-rf" | "-fr" | "-Rf" | "-fR"))
    }

    fn targets_root(args: &[String]) -> Option<&'static str> {
        ROOT_TARGETS
            .iter()
            .find(|&&root| {
                args.iter().any(|a| {
                    // Strip trailing slashes before comparing so "/etc/" == "/etc"
                    a.trim_end_matches('/') == root.trim_end_matches('/')
                })
            })
            .copied()
    }
}

impl Guard for RmGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        if cmd.name != "rm" {
            return Ok(());
        }

        let args = token_args(&cmd.args);

        // Bloquea si: recursivo + forzado + target es raíz
        if Self::has_recursive(&args) && Self::has_force(&args) {
            if let Some(target) = Self::targets_root(&args) {
                return Err(GuardError::DestructiveFs {
                    reason: t!("guard.destructive_fs.rm_root_blocked", target = target),
                });
            }
        }
        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════
// ChmodChownGuard — El Anarquista
// ══════════════════════════════════════════════════════════════

pub struct ChmodChownGuard;

impl ChmodChownGuard {
    pub fn new() -> Self {
        Self
    }

    const PROTECTED_PATHS: &'static [&'static str] = &["/", "/usr", "/usr/*", "/etc", "/etc/*"];
}

impl Guard for ChmodChownGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        if !matches!(cmd.name.as_str(), "chmod" | "chown") {
            return Ok(());
        }

        let args = token_args(&cmd.args);
        let has_recursive = args
            .iter()
            .any(|a| matches!(a.as_str(), "-R" | "--recursive"));

        if has_recursive {
            let targets_protected = Self::PROTECTED_PATHS
                .iter()
                .any(|&p| args.iter().any(|a| a == p));
            if targets_protected {
                return Err(GuardError::DestructiveFs {
                    reason: t!("guard.destructive_fs.chmod_protected_blocked", cmd = cmd.name),
                });
            }
        }
        Ok(())
    }
}

// Helper compartido
pub(super) fn token_args(tokens: &[Token]) -> Vec<String> {
    tokens
        .iter()
        .filter_map(|t| t.as_str().map(str::to_owned))
        .collect()
}
