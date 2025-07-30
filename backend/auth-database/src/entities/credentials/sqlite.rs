// #[cfg(feature = "unit")]
use crate::entities::credentials::{
    CreateCredentialsDAO, CredentialsBy, CredentialsDAO, CredentialsWhere, UpdateCredentialsDAO,
};

use database::traits::{DatabaseError, EntityRepository};
use sqlx::{Transaction, types::Uuid};

use std::str::FromStr;

// #[cfg(feature = "unit")]
use sqlx::Sqlite;

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct SqliteCredentialsDAO {
    pub id: String,
    pub email: String,
    pub password: String,
    pub active: bool,
}

impl From<CredentialsDAO> for SqliteCredentialsDAO {
    fn from(value: CredentialsDAO) -> Self {
        SqliteCredentialsDAO {
            id: value.id.to_string(),
            email: value.email,
            password: value.password,
            active: value.active,
        }
    }
}

impl TryFrom<SqliteCredentialsDAO> for CredentialsDAO {
    type Error = DatabaseError;
    fn try_from(value: SqliteCredentialsDAO) -> Result<Self, Self::Error> {
        Ok(CredentialsDAO {
            id: Uuid::from_str(&value.id)
                .map_err(|_| DatabaseError::Unknown("Could not convert id to uuid".to_string()))?,
            email: value.email,
            password: value.password,
            active: value.active,
        })
    }
}

#[derive(Debug)]
pub struct SqliteCredentialsRepository;

// #[cfg(feature = "unit")]
#[database::async_trait::async_trait]
impl EntityRepository for SqliteCredentialsRepository {
    type Db = Sqlite;
    type Entity = CredentialsDAO;
    type CreateInput = CreateCredentialsDAO;
    type UpdateInput = UpdateCredentialsDAO;
    type QueryOne = CredentialsBy;
    type QueryMany = CredentialsWhere;

    async fn exists(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<bool, DatabaseError> {
        Ok(SqliteCredentialsRepository::try_get(tx, key)
            .await?
            .is_some())
    }

    async fn insert(
        tx: &mut Transaction<'_, Self::Db>,
        input: Self::CreateInput,
    ) -> Result<Self::Entity, DatabaseError> {
        let credential = sqlx::query_as::<_, SqliteCredentialsDAO>(
            "INSERT INTO credentials (id, email, password) VALUES ($1, $2, $3) RETURNING id, email, password, active;",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(input.email)
        .bind(input.password)
        .fetch_one(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

        Ok(Self::Entity::try_from(credential)?)
    }

    async fn delete(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Self::Entity, DatabaseError> {
        let credential = match key {
            CredentialsBy::Id(uuid) => {
                sqlx::query_as::<_, SqliteCredentialsDAO>("UPDATE credentials SET active = false WHERE id = $1 RETURNING id, email, password, active;")
                    .bind(uuid.to_string())
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)?
            },
            CredentialsBy::Email(email) => {
                sqlx::query_as::<_, SqliteCredentialsDAO>("UPDATE credentials SET active = false WHERE email = $1 RETURNING id, password, email, active;")
                    .bind(email)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(DatabaseError::from)?
            },

        };

        Ok(Self::Entity::try_from(credential)?)
    }

    async fn update(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
        update: Self::UpdateInput,
    ) -> Result<Self::Entity, DatabaseError> {
        let crendential = match key {
            CredentialsBy::Id(id) => sqlx::query_as::<_, SqliteCredentialsDAO>(
                "UPDATE credentials SET password = $2, active = $3 WHERE id = $1 RETURNING id, email, password, active;",
            )
                .bind(id.to_string())
                .bind(update.password)
                .bind(update.active)
                .fetch_one(&mut **tx)
                .await
                .map_err(DatabaseError::from)?,
            CredentialsBy::Email(email) => sqlx::query_as::<_, SqliteCredentialsDAO>(
                "UPDATE credentials SET password = $2, active = $3 WHERE email = $1 RETURNING id, email, password, active;",
            )
                .bind(email.to_string())
                .bind(update.password)
                .bind(update.active)
                .fetch_one(&mut **tx)
                .await
                .map_err(DatabaseError::from)?,
        };

        Ok(Self::Entity::try_from(crendential)?)
    }

    async fn get(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Self::Entity, DatabaseError> {
        let credential = match key {
            CredentialsBy::Id(id) => sqlx::query_as::<_, SqliteCredentialsDAO>(
                "SELECT id, email, password, active FROM credentials WHERE id = $1 LIMIT 1;",
            )
            .bind(id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(DatabaseError::from)?,
            CredentialsBy::Email(email) => sqlx::query_as::<_, SqliteCredentialsDAO>(
                "SELECT id, email, password, active FROM credentials WHERE email = $1 LIMIT 1;",
            )
            .bind(email)
            .fetch_one(&mut **tx)
            .await
            .map_err(DatabaseError::from)?,
        };

        Ok(Self::Entity::try_from(credential)?)
    }

    async fn try_get(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Option<Self::Entity>, DatabaseError> {
        let maybe_credential = match key {
            CredentialsBy::Id(uuid) => sqlx::query_as::<_, SqliteCredentialsDAO>(
                "SELECT id, email, password, active FROM credentials WHERE id = $1;",
            )
            .bind(uuid.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(DatabaseError::from)?,
            CredentialsBy::Email(email) => sqlx::query_as::<_, SqliteCredentialsDAO>(
                "SELECT id, email, password, active FROM credentials WHERE email = $1;",
            )
            .bind(email)
            .fetch_optional(&mut **tx)
            .await
            .map_err(DatabaseError::from)?,
        };

        if let Some(credential) = maybe_credential {
            Ok(Some(Self::Entity::try_from(credential)?))
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
}
