use sqlx::{Database, Transaction, types::Uuid};

use crate::server::ServerError;

#[derive(sqlx::FromRow, PartialEq, Eq, Clone)]
pub struct CredentialDAO {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub active: bool,
}

pub struct CreateCredentialDAO {
    pub email: String,
    pub password: String,
}

#[async_trait::async_trait]
pub trait CredentialsEntityRepository {
    type Db: Database;
    type Error: Into<ServerError>;

    async fn exists(tx: &mut Transaction<'_, Self::Db>, email: &str) -> Result<bool, Self::Error>;
    async fn create(
        tx: &mut Transaction<'_, Self::Db>,
        credential: CreateCredentialDAO,
    ) -> Result<CredentialDAO, Self::Error>;
}
