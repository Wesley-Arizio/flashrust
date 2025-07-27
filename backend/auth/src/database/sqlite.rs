use std::str::FromStr;

use sqlx::error::BoxDynError;
#[cfg(feature = "unit")]
use sqlx::{Sqlite, Transaction};

use uuid::Uuid;

use crate::database::traits::CredentialDAO;

#[cfg(feature = "unit")]
use crate::database::traits::{CreateCredentialDAO, CredentialsEntityRepository};

#[derive(sqlx::FromRow, PartialEq, Eq, Clone)]
struct SqliteCredentialDAO {
    pub id: String,
    pub email: String,
    pub password: String,
    pub active: bool,
}

impl TryFrom<SqliteCredentialDAO> for CredentialDAO {
    type Error = sqlx::Error;
    fn try_from(value: SqliteCredentialDAO) -> Result<Self, Self::Error> {
        Ok(CredentialDAO {
            id: Uuid::from_str(&value.id)
                .map_err(|e| sqlx::Error::Decode(Box::new(e) as BoxDynError))?,
            email: value.email,
            password: value.password,
            active: value.active,
        })
    }
}

pub struct SqliteCredentialsRepository {}

#[cfg(feature = "unit")]
#[async_trait::async_trait]
impl CredentialsEntityRepository for SqliteCredentialsRepository {
    type Db = Sqlite;
    type Error = sqlx::Error;

    async fn exists(tx: &mut Transaction<'_, Self::Db>, email: &str) -> Result<bool, Self::Error> {
        let result = sqlx::query_as::<_, (bool,)>(
            "SELECT EXISTS (SELECT 1 FROM credentials WHERE email = ?)",
        )
        .bind(email)
        .fetch_one(&mut **tx)
        .await?;
        Ok(result.0)
    }

    async fn create(
        tx: &mut Transaction<'_, Self::Db>,
        credential: CreateCredentialDAO,
    ) -> Result<CredentialDAO, Self::Error> {
        use sqlx::types::Uuid;

        let result = sqlx::query_as::<_, SqliteCredentialDAO>("INSERT INTO credentials (id, email, password) VALUES ($1, $2, $3) RETURNING id, email, password, active;")
            .bind(Uuid::new_v4().to_string())
            .bind(credential.email)
            .bind(credential.password)
            .fetch_one(&mut **tx)
            .await?;

        Ok(result.try_into()?)
    }
}
