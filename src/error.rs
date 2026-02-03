use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpmError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IDE not found: {0}")]
    IdeNotFound(String),

    #[error("IDE config not found: {0}")]
    IdeConfigNotFound(String),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Installation failed: {0}")]
    InstallFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, McpmError>;
