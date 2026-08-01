use serde::Serialize;
use thiserror::Error;

use crate::domain::ValidationIssue;

#[derive(Debug, Clone, Error, Serialize)]
#[serde(tag = "code", content = "details", rename_all = "camelCase")]
pub enum AppError {
    #[error("settings validation failed")]
    InvalidSettings(Vec<ValidationIssue>),
    #[error("local state could not be read or written: {0}")]
    Persistence(String),
    #[error("a native notification could not be sent: {0}")]
    Notification(String),
    #[error("a window operation failed: {0}")]
    Window(String),
    #[error("launch-at-login could not be changed: {0}")]
    Autostart(String),
    #[error("application state is temporarily unavailable")]
    StateUnavailable,
}
