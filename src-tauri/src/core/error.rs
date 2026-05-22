use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    PathNotFound,
    PermissionDenied,
    FileInUse,
    TargetAlreadyExists,
    RecycleBinUnavailable,
    DefaultAppOpenFailed,
    TerminalUnavailable,
    PropertiesOpenFailed,
    CacheUnwritable,
    TestRootEscape,
    InternalError,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    pub refresh_suggestion: Option<String>,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: false,
            refresh_suggestion: None,
        }
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub fn with_refresh(mut self, suggestion: impl Into<String>) -> Self {
        self.refresh_suggestion = Some(suggestion.into());
        self
    }
}
