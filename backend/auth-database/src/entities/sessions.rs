pub mod postgres;

#[cfg(feature = "unit")]
pub mod sqlite;

use sqlx::types::Uuid;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct SessionsDAO {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub credential_id: Uuid,
    pub active: bool,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct CreateSessionsDAO {
    pub expires_at: DateTime<Utc>,
    pub credential_id: Uuid,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct UpdateSessionsDAO {}

#[derive(Debug, PartialEq, Eq)]
pub enum SessionsBy {
    Id(Uuid),
    CredentialId(Uuid),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SessionsWhere {
    CredentialId(Uuid),
}
