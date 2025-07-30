use crate::entities::sessions::{
    CreateSessionsDAO, SessionsBy, SessionsDAO, SessionsWhere, UpdateSessionsDAO,
};
use database::traits::{DatabaseError, EntityRepository};
use sqlx::types::Uuid;
use sqlx::types::chrono::DateTime;

use sqlx::{Sqlite, Transaction};
use std::str::FromStr;

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct SqliteSessionsDAO {
    pub id: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub credential_id: String,
    pub active: bool,
}

impl TryFrom<SqliteSessionsDAO> for SessionsDAO {
    type Error = DatabaseError;
    fn try_from(value: SqliteSessionsDAO) -> Result<Self, DatabaseError> {
        Ok(SessionsDAO {
            id: Uuid::from_str(&value.id)
                .map_err(|_| DatabaseError::Unknown("Could not convert id to uuid".to_string()))?,
            created_at: DateTime::from_timestamp_millis(value.created_at).ok_or(
                DatabaseError::Unknown("Could not convert created_at to DateTime<Utc>".to_string()),
            )?,
            expires_at: DateTime::from_timestamp_millis(value.expires_at).ok_or(
                DatabaseError::Unknown("Could not convert expires_at to DateTime<Utc>".to_string()),
            )?,
            credential_id: Uuid::from_str(&value.credential_id).map_err(|_| {
                DatabaseError::Unknown("Could not convert credential_id to uuid".to_string())
            })?,
            active: value.active,
        })
    }
}

impl From<SessionsDAO> for SqliteSessionsDAO {
    fn from(value: SessionsDAO) -> Self {
        SqliteSessionsDAO {
            id: value.id.to_string(),
            created_at: value.created_at.timestamp_millis(),
            expires_at: value.expires_at.timestamp_millis(),
            credential_id: value.credential_id.to_string(),
            active: value.active,
        }
    }
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct SqliteCreateSessionsDAO {
    pub expires_at: i64,
    pub credential_id: String,
}

impl From<CreateSessionsDAO> for SqliteCreateSessionsDAO {
    fn from(value: CreateSessionsDAO) -> Self {
        SqliteCreateSessionsDAO {
            expires_at: value.expires_at.timestamp_millis(),
            credential_id: value.credential_id.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct SqliteSessionsRepository;

#[database::async_trait::async_trait]
impl EntityRepository for SqliteSessionsRepository {
    type Db = Sqlite;
    type Entity = SessionsDAO;
    type CreateInput = CreateSessionsDAO;
    type UpdateInput = UpdateSessionsDAO;
    type QueryOne = SessionsBy;
    type QueryMany = SessionsWhere;

    async fn insert(
        tx: &mut Transaction<'_, Self::Db>,
        input: Self::CreateInput,
    ) -> Result<Self::Entity, DatabaseError> {
        let input: SqliteCreateSessionsDAO = input.into();
        let result = sqlx::query_as::<_, SqliteSessionsDAO>("INSERT INTO sessions (id, expires_at, credential_id) VALUES ($1, $2, $3) RETURNING id, created_at, expires_at, credential_id, active;")
            .bind(Uuid::new_v4().to_string())
            .bind(input.expires_at)
            .bind(input.credential_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(DatabaseError::from)?;

        Ok(Self::Entity::try_from(result)?)
    }

    async fn delete(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Self::Entity, DatabaseError> {
        let session = match key {
            SessionsBy::Id(uuid) => {
                sqlx::query_as::<_, SqliteSessionsDAO>("UPDATE sessions SET active = false WHERE id = $1 RETURNING id, created_at, expires_at, credential_id, active;")
                    .bind(uuid.to_string())
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)?
            },
            SessionsBy::CredentialId(uuid) => {
                sqlx::query_as::<_, SqliteSessionsDAO>("UPDATE sessions SET active = false WHERE credential_id = $1 RETURNING id, created_at, expires_at, credential_id, active;")
                    .bind(uuid.to_string())
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)?
            },
        };

        Ok(Self::Entity::try_from(session)?)
    }

    async fn update(
        _tx: &mut Transaction<'_, Self::Db>,
        _key: Self::QueryOne,
        _update: Self::UpdateInput,
    ) -> Result<Self::Entity, DatabaseError> {
        unreachable!("")
    }

    async fn get(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Self::Entity, DatabaseError> {
        let session = match key {
            SessionsBy::Id(id) => sqlx::query_as::<_, SqliteSessionsDAO>(
                "SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE id = $1 LIMIT 1;",
            )
                .bind(id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(DatabaseError::from)?,
            SessionsBy::CredentialId(uuid) => sqlx::query_as::<_, SqliteSessionsDAO>(
                "SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE credential_id = $1 LIMIT 1;",
            )
                .bind(uuid.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(DatabaseError::from)?,
        };

        Ok(Self::Entity::try_from(session)?)
    }

    async fn try_get(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Option<Self::Entity>, DatabaseError> {
        let maybe_session = match key {
            SessionsBy::Id(uuid) => {
                sqlx::query_as::<_, SqliteSessionsDAO>("SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE id = $1 LIMIT 1;")
                    .bind(uuid.to_string())
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)?
            },
            SessionsBy::CredentialId(uuid) => {
                sqlx::query_as("SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE credential_id = $1 LIMIT 1;")
                    .bind(uuid.to_string())
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)?
            }
        };

        if let Some(s) = maybe_session {
            Ok(Some(Self::Entity::try_from(s)?))
        } else {
            Ok(None)
        }
    }

    async fn get_all(
        _tx: &mut Transaction<'_, Self::Db>,
        _key: Self::QueryMany,
    ) -> Result<Vec<Self::Entity>, DatabaseError> {
        todo!()
    }

    async fn exists(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<bool, DatabaseError> {
        Ok(SqliteSessionsRepository::try_get(tx, key).await?.is_some())
    }
}
