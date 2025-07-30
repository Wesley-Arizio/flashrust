use database::traits::{DatabaseError, EntityRepository};
use sqlx::{Postgres, Transaction};

use crate::entities::credentials::{
    CreateCredentialsDAO, CredentialsBy, CredentialsDAO, CredentialsWhere, UpdateCredentialsDAO,
};

#[derive(Debug)]
pub struct PostgresCredentialsRepository;

#[database::async_trait::async_trait]
impl EntityRepository for PostgresCredentialsRepository {
    type Db = Postgres;
    type Entity = CredentialsDAO;
    type CreateInput = CreateCredentialsDAO;
    type UpdateInput = UpdateCredentialsDAO;
    type QueryOne = CredentialsBy;
    type QueryMany = CredentialsWhere;

    async fn insert(
        tx: &mut Transaction<'_, Self::Db>,
        input: Self::CreateInput,
    ) -> Result<Self::Entity, DatabaseError> {
        sqlx::query_as::<_, Self::Entity>("INSERT INTO credentials (email, password) VALUES ($1, $2) RETURNING id, email, password, active;")
            .bind(input.email)
            .bind(input.password)
            .fetch_one(&mut **tx)
            .await
            .map_err(DatabaseError::from)
    }

    async fn delete(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Self::Entity, DatabaseError> {
        match key {
            CredentialsBy::Id(uuid) => {
                sqlx::query_as::<_, Self::Entity>("UPDATE credentials SET active = false WHERE id = $1 RETURNING id, email, password, active;")
                    .bind(uuid)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)
            },
            CredentialsBy::Email(email) => {
                sqlx::query_as::<_, Self::Entity>("UPDATE credentials SET active = false WHERE email = $1 RETURNING id, password, email, active;")
                    .bind(email)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)
            },

        }
    }

    async fn update(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
        update: Self::UpdateInput,
    ) -> Result<Self::Entity, DatabaseError> {
        match key {
            CredentialsBy::Id(id) => sqlx::query_as::<_, Self::Entity>(
                "UPDATE credentials SET password = $2, active = $3 WHERE id = $1 RETURNING id, email, password, active;",
            )
                .bind(id)
                .bind(update.password)
                .bind(update.active)
                .fetch_one(&mut **tx)
                .await
                .map_err(DatabaseError::from),
            CredentialsBy::Email(email) => sqlx::query_as::<_, Self::Entity>(
                "UPDATE credentials SET password = $2, active = $3 WHERE email = $1 RETURNING id, email, password, active;",
            )
                .bind(email)
                .bind(update.password)
                .bind(update.active)
                .fetch_one(&mut **tx)
                .await
                .map_err(DatabaseError::from),
        }
    }

    async fn get(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Self::Entity, DatabaseError> {
        match key {
            CredentialsBy::Id(id) => sqlx::query_as::<_, Self::Entity>(
                "SELECT id, email, password, active FROM credentials WHERE id = $1 LIMIT 1;",
            )
            .bind(id)
            .fetch_one(&mut **tx)
            .await
            .map_err(DatabaseError::from),
            CredentialsBy::Email(email) => sqlx::query_as::<_, Self::Entity>(
                "SELECT id, email, password, active FROM credentials WHERE email = $1 LIMIT 1;",
            )
            .bind(email)
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
            CredentialsBy::Id(uuid) => {
                sqlx::query_as("SELECT id, email, password, active FROM credentials WHERE id = $1;")
                    .bind(uuid)
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)
            }
            CredentialsBy::Email(email) => sqlx::query_as(
                "SELECT id, email, password, active FROM credentials WHERE email = $1;",
            )
            .bind(email)
            .fetch_optional(&mut **tx)
            .await
            .map_err(DatabaseError::from),
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
        Ok(PostgresCredentialsRepository::try_get(tx, key)
            .await?
            .is_some())
    }
}
