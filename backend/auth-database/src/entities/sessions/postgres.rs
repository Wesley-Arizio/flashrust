use crate::entities::sessions::{
    CreateSessionsDAO, SessionsBy, SessionsDAO, SessionsWhere, UpdateSessionsDAO,
};
use database::traits::{DatabaseError, EntityRepository};
use sqlx::{Postgres, Transaction};

#[derive(Debug)]
pub struct PostgresSessionsRepository;

#[database::async_trait::async_trait]
impl EntityRepository for PostgresSessionsRepository {
    type Db = Postgres;
    type Entity = SessionsDAO;
    type CreateInput = CreateSessionsDAO;
    type UpdateInput = UpdateSessionsDAO;
    type QueryOne = SessionsBy;
    type QueryMany = SessionsWhere;

    async fn insert(
        tx: &mut Transaction<'_, Self::Db>,
        input: Self::CreateInput,
    ) -> Result<Self::Entity, DatabaseError> {
        sqlx::query_as::<_, Self::Entity>("INSERT INTO sessions (expires_at, credential_id) VALUES ($1, $2) RETURNING id, created_at, expires_at, credential_id, active;")
            .bind(input.expires_at)
            .bind(input.credential_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(DatabaseError::from)
    }

    async fn delete(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Self::Entity, DatabaseError> {
        match key {
            SessionsBy::Id(uuid) => {
                sqlx::query_as::<_, Self::Entity>("UPDATE sessions SET active = false WHERE id = $1 RETURNING id, created_at, expires_at, credential_id, active;")
                    .bind(uuid)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)
            },
            SessionsBy::CredentialId(uuid) => {
                sqlx::query_as::<_, Self::Entity>("UPDATE sessions SET active = false WHERE credential_id = $1 RETURNING id, created_at, expires_at, credential_id, active;")
                    .bind(uuid)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)
            },

        }
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
        match key {
            SessionsBy::Id(id) => sqlx::query_as::<_, Self::Entity>(
                "SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE id = $1 LIMIT 1;",
            )
                .bind(id)
                .fetch_one(&mut **tx)
                .await
                .map_err(DatabaseError::from),
            SessionsBy::CredentialId(uuid) => sqlx::query_as::<_, Self::Entity>(
                "SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE credential_id = $1 LIMIT 1;",
            )
                .bind(uuid)
                .fetch_one(&mut **tx)
                .await
                .map_err(DatabaseError::from),
        }
    }

    async fn try_get(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Option<Self::Entity>, DatabaseError> {
        match key {
            SessionsBy::Id(uuid) => {
                sqlx::query_as("SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE id = $1 LIMIT 1;")
                    .bind(uuid)
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)
            },
            SessionsBy::CredentialId(uuid) => {
                sqlx::query_as("SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE credential_id = $1 LIMIT 1;")
                    .bind(uuid)
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)
            }
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
        Ok(PostgresSessionsRepository::try_get(tx, key)
            .await?
            .is_some())
    }
}
