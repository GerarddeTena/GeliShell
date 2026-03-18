#[derive(Debug, thiserror::Error)]
pub enum ShowMeError {
    #[error("docs.db not found at {path}")]
    DbNotFound { path: String },

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("terminal error: {0}")]
    Terminal(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("catalog is empty - no documents indexed")]
    EmptyCatalog,
}
