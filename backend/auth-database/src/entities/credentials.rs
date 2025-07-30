use sqlx::types::Uuid;

pub mod postgres;

#[cfg(feature = "unit")]
pub mod sqlite;

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct CredentialsDAO {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub active: bool,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct CreateCredentialsDAO {
    pub email: String,
    pub password: String,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct UpdateCredentialsDAO {
    pub password: String,
    pub active: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CredentialsBy {
    Id(Uuid),
    Email(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CredentialsWhere {
    Active(bool),
}
