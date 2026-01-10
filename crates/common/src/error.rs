//! Common error types.

use thiserror::Error;

/// Main error type for the browser engine.
#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("JavaScript error: {0}")]
    JavaScript(String),

    #[error("Layout error: {0}")]
    Layout(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type BrowserResult<T> = Result<T, BrowserError>;

impl BrowserError {
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    pub fn js(msg: impl Into<String>) -> Self {
        Self::JavaScript(msg.into())
    }

    pub fn layout(msg: impl Into<String>) -> Self {
        Self::Layout(msg.into())
    }

    pub fn render(msg: impl Into<String>) -> Self {
        Self::Render(msg.into())
    }

    pub fn security(msg: impl Into<String>) -> Self {
        Self::Security(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::InvalidOperation(msg.into())
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
