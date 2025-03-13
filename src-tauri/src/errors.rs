use crate::atproto_oauth::ParResponseError;
use serde_path_to_error::Error as SerdePathError;
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
    #[error("Reqwest error: {0}, {__display0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    ParseError(#[from] oauth2::url::ParseError),
    #[error(
        "{classify:?} error parsing JSON: (expected: {message}) on line {line} column {column}"
    )]
    JsonError {
        message: String,
        classify: serde_json::error::Category,
        column: usize,
        line: usize,
    },
    #[error("PAR Error: {error}. {description}")]
    ParError { error: String, description: String },
    #[error("No answer found in DNS response")]
    NoAnswerFound(u8),
    #[error("{0}")]
    ValidationError(String),
    #[error("{err_type}: {message}")]
    AuthError { message: String, err_type: String },
}
impl serde::Serialize for MyError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
impl From<serde_json::Error> for MyError {
    fn from(error: serde_json::Error) -> Self {
        MyError::JsonError {
            message: error.to_string(),
            classify: error.classify(),
            column: error.column(),
            line: error.line(),
        }
    }
}
impl From<ParResponseError> for MyError {
    fn from(json: ParResponseError) -> Self {
        MyError::ParError {
            error: json.error,
            description: json.error_description,
        }
    }
}

#[derive(Debug)]
pub struct SerializationError {
    path: String,
    cause: String,
}


impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "at path '{}': {}", self.path, self.cause)
    }
}

impl std::error::Error for SerializationError {}

impl From<SerdePathError<reqwest::Error>> for SerializationError {
    fn from(err: SerdePathError<reqwest::Error>) -> Self {
        let path = err.path().to_string();
        let cause = err.inner().to_string();

        SerializationError { path, cause }
    }
}
