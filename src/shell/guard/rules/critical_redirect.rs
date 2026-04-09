use crate::parser::ast::Command;
use crate::parser::token::RedirectKind;
use crate::shell::guard::Guard;
use crate::shell::guard::error::GuardError;
use crate::t;

/// Archivos críticos que nunca deben ser sobreescritos con >
const CRITICAL_FILES: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/sudoers",
    "/etc/hosts",
    "/etc/fstab",
    "/etc/crontab",
    "/boot/grub/grub.cfg",
    "/proc/sysrq-trigger",
    "/dev/sda",
    "/dev/sdb",
    "/dev/nvme0n1",
];

pub struct CriticalRedirectGuard;

impl CriticalRedirectGuard {
    pub fn new() -> Self {
        Self
    }
}

impl Guard for CriticalRedirectGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        for redir in &cmd.redirections {
            // Solo bloquea > (Out) y >> (Append) — no < (In)
            if !matches!(redir.kind, RedirectKind::Out | RedirectKind::Append) {
                continue;
            }

            let target = match &redir.target {
                crate::parser::token::Token::Word(s) => s.clone(),
                _ => continue,
            };

            // Bloquea /proc/sysrq-trigger explícitamente — Kernel Panic
            if target == "/proc/sysrq-trigger" {
                return Err(GuardError::CriticalRedirect {
                    reason: t!("guard.critical_redirect.sysrq_blocked"),
                });
            }

            // Bloquea archivos críticos del sistema
            if CRITICAL_FILES.iter().any(|&f| target == f) {
                return Err(GuardError::CriticalRedirect {
                    reason: t!(
                        "guard.critical_redirect.system_file_blocked",
                        target = target
                    ),
                });
            }
        }
        Ok(())
    }
}
