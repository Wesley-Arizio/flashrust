use database::traits::{BaseDatabase, DatabaseError};
use sqlx::{Database, Pool};

#[cfg(not(feature = "unit"))]
use sqlx::PgPool;

pub mod entities;

#[cfg(feature = "unit")]
use sqlx::SqlitePool;

#[cfg(feature = "unit")]
pub use crate::entities::credentials::sqlite::SqliteCredentialsRepository as CredentialsRepository;

#[cfg(feature = "unit")]
pub use crate::entities::sessions::sqlite::SqliteSessionsRepository as SessionsRepository;

#[cfg(not(feature = "unit"))]
pub use crate::entities::credentials::postgres::PostgresCredentialsRepository as CredentialsRepository;

#[cfg(not(feature = "unit"))]
pub use crate::entities::sessions::postgres::PostgresSessionsRepository as SessionsRepository;

pub use database::*;

#[cfg(feature = "unit")]
pub type DB = sqlx::Sqlite;

#[cfg(not(feature = "unit"))]
pub type DB = sqlx::Postgres;

pub struct AuthDatabase;

impl<DB: Database> BaseDatabase<DB> for AuthDatabase {}

impl AuthDatabase {
    pub async fn connect(url: &str) -> Result<Pool<DB>, DatabaseError> {
        #[cfg(feature = "unit")]
        {
            let pool = SqlitePool::connect(url).await?;
            sqlx::migrate!("./sqlite")
                .run(&pool)
                .await
                .expect("Failed to run sqlite migrations");
            Ok(pool)
        }

        #[cfg(not(feature = "unit"))]
        {
            let pool = PgPool::connect(url).await?;
            Ok(pool)
        }
    }
}
