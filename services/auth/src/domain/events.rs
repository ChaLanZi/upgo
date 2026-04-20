use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub enum AuthEvent {
    UserLoggedIn {
        user_id: Uuid,
        session_id: Uuid,
        platform: String,
        timestamp: DateTime<Utc>,
    },
    UserRegistered {
        user_id: Uuid,
        email: String,
        timestamp: DateTime<Utc>,
    },
    UserLoggedOut {
        user_id: Uuid,
        session_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    PasswordChanged {
        user_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    EmailChanged {
        user_id: Uuid,
        old_email: String,
        new_email: String,
        timestamp: DateTime<Utc>,
    },
    UserDeleted {
        user_id: Uuid,
        soft_deleted_at: DateTime<Utc>,
        permanent_delete_at: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },
}
