/// Represents an authenticated user session stored in-memory in `AppState`.
#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: String,
    pub role: String,
    pub language: String,
    pub family_id: String,
}
