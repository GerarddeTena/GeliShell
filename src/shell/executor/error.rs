use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("process was killed by signal")]
    KilledBySignal,

    #[error("command string is empty")]
    EmptyCommand,

    #[error("timeout after {0}s")]
    Timeout(u64),
}