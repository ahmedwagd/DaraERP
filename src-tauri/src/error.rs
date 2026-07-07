use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
    pub code: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        match &e {
            rusqlite::Error::SqliteFailure(err, _) => {
                if err.code == rusqlite::ErrorCode::ConstraintViolation {
                    Self::new("DB_CONSTRAINT", "A database constraint was violated")
                } else {
                    Self::new("DATABASE_ERROR", "A database error occurred")
                }
            }
            _ => Self::new("DATABASE_ERROR", "A database error occurred"),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::new("SERIALIZATION_ERROR", e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::new("IO_ERROR", e.to_string())
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::new("INTERNAL_ERROR", s)
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::new("INTERNAL_ERROR", s)
    }
}
