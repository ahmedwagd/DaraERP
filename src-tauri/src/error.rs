use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
    pub code: String,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError {
            message: e.to_string(),
            code: "DB_ERROR".into(),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError {
            message: e.to_string(),
            code: "SERIALIZATION_ERROR".into(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError {
            message: e.to_string(),
            code: "IO_ERROR".into(),
        }
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError {
            message: s.to_string(),
            code: "GENERAL_ERROR".into(),
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError {
            message: s,
            code: "GENERAL_ERROR".into(),
        }
    }
}
