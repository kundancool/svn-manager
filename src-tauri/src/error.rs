use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("svn binary not found")]
    SvnNotFound,
    #[error("not a working copy: {0}")]
    NotAWorkingCopy(String),
    #[error("authentication required for {realm}")]
    AuthRequired { realm: String },
    #[error("svn command failed: {stderr}")]
    SvnCommand { stderr: String },
    #[error("failed to parse svn output: {0}")]
    Parse(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(String),
}

// Tauri commands need serializable errors; serialize as { kind, message }.
#[derive(Serialize)]
struct ErrorPayload {
    kind: &'static str,
    message: String,
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let kind = match self {
            AppError::SvnNotFound => "svn_not_found",
            AppError::NotAWorkingCopy(_) => "not_a_working_copy",
            AppError::AuthRequired { .. } => "auth_required",
            AppError::SvnCommand { .. } => "svn_command",
            AppError::Parse(_) => "parse",
            AppError::Io(_) => "io",
            AppError::Config(_) => "config",
        };
        ErrorPayload { kind, message: self.to_string() }.serialize(serializer)
    }
}

pub type AppResult<T> = Result<T, AppError>;
