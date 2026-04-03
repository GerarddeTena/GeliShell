use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum GuardError {
    #[error("🔴 BLOCKED — destructive filesystem operation: {reason}")]
    DestructiveFs { reason: String },

    #[error("🔴 BLOCKED — disk destroyer detected: {reason}")]
    DiskDestroyer { reason: String },

    #[error("🔴 BLOCKED — critical file redirect: {reason}")]
    CriticalRedirect { reason: String },

    #[error("🔴 BLOCKED — pipe execution from network: {reason}")]
    PipeExecution { reason: String },

    #[error("🔴 BLOCKED — fork bomb pattern detected")]
    ForkBomb,

    #[error("🔴 BLOCKED — blacklisted command: '{name} {args}'",
        args = .args.join(" "))]
    BlacklistedCommand { name: String, args: Vec<String> },

    #[error("🔴 BLOCKED — forbidden argument '{arg}' in '{command}'")]
    ForbiddenArgument { command: String, arg: String },

    #[error("⚠️  CONFIRMATION REQUIRED — {reason}")]
    RequiresConfirmation { reason: String },
}

impl GuardError {
    pub fn is_fatal(&self) -> bool {
        !matches!(self, Self::RequiresConfirmation { .. })
    }
}