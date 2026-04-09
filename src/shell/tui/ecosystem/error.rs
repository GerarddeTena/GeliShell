#[derive(Debug, thiserror::Error)]
pub enum EcosystemTuiError {
    #[error("terminal error: {0}")]
    Terminal(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
