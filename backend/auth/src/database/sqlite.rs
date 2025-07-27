#[cfg(feature = "unit")]
use sqlx::types::Uuid;

#[cfg(feature = "unit")]
use sqlx::{Sqlite, Transaction};

#[cfg(feature = "unit")]
use crate::database::traits::{CreateCredentialDAO, CredentialDAO, CredentialsEntityRepository};

pub struct SqliteCredentialsRepository {}

#[cfg(feature = "unit")]
#[async_trait::async_trait]
impl CredentialsEntityRepository for SqliteCredentialsRepository {
    type Db = Sqlite;
    type Error = String;

    async fn exists(_email: &str) -> Result<bool, Self::Error> {
        todo!("")
    }

    async fn create(
        _tx: &mut Transaction<'_, Self::Db>,
        _credential: CreateCredentialDAO,
    ) -> Result<CredentialDAO, Self::Error> {
        todo!("")
    }
}
