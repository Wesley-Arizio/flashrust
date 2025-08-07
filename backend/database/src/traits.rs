use sqlx::{Database, Error as SqlxError, Pool, Transaction};
use std::{fmt, pin::Pin};

#[derive(Debug)]
pub enum DatabaseError {
    NotFound(String),
    CommunicationError,
    ConnectionFailed,
    ConnectionNotAvailable,
    QueryFailed(String),
    ColumnNotFound(String),
    ProtocolNotSupported,
    NotImplemented,
    Unknown(String),
    DatabaseInconsistence(String),
    MigrationFailed(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::NotFound(msg) => write!(f, "Not Found: {msg}"),
            DatabaseError::CommunicationError => write!(f, "Communication Error"),
            DatabaseError::ConnectionFailed => write!(f, "Connection Failed"),
            DatabaseError::ConnectionNotAvailable => write!(f, "Connection Not Available"),
            DatabaseError::QueryFailed(msg) => write!(f, "Query Failed: {msg}"),
            DatabaseError::ColumnNotFound(column) => write!(f, "Column Not Found: {column}"),
            DatabaseError::ProtocolNotSupported => write!(f, "Protocol Not Supported"),
            DatabaseError::NotImplemented => write!(f, "Not Implemented"),
            DatabaseError::Unknown(msg) => write!(f, "Unknown Error: {msg}"),
            DatabaseError::DatabaseInconsistence(msg) => {
                write!(f, "Database Inconsistency: {msg}")
            }
            DatabaseError::MigrationFailed(msg) => write!(f, "Migration Failed: {msg}"),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Since DatabaseError doesn't wrap other errors in the current implementation,
        // we return None. If you add error chaining later, update this to return the source.
        None
    }
}

impl From<SqlxError> for DatabaseError {
    fn from(value: SqlxError) -> Self {
        match value {
            SqlxError::ColumnNotFound(column_name) => Self::ColumnNotFound(column_name),
            SqlxError::Io(_) | SqlxError::Tls(_) => Self::CommunicationError,
            SqlxError::PoolTimedOut => Self::ConnectionNotAvailable,
            SqlxError::Database(e) => Self::QueryFailed(e.to_string()),
            SqlxError::Protocol(_) => Self::ProtocolNotSupported,
            SqlxError::TypeNotFound { type_name } => {
                Self::DatabaseInconsistence(format!("TypeNotFound {type_name}"))
            }
            _ => Self::ConnectionFailed,
        }
    }
}

#[async_trait::async_trait]
pub trait EntityRepository {
    type Db: Database;
    type Entity: Send;
    type CreateInput: Send;
    type UpdateInput: Send;
    type QueryOne: Send + Sync;
    type QueryMany: Send + Sync;
    async fn insert(
        tx: &mut Transaction<'_, Self::Db>,
        input: Self::CreateInput,
    ) -> Result<Self::Entity, DatabaseError>;
    async fn delete(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Self::Entity, DatabaseError>;
    async fn update(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
        update: Self::UpdateInput,
    ) -> Result<Self::Entity, DatabaseError>;
    async fn get(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Self::Entity, DatabaseError>;
    async fn try_get(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<Option<Self::Entity>, DatabaseError>;
    async fn get_all(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryMany,
    ) -> Result<Vec<Self::Entity>, DatabaseError>;

    async fn exists(
        tx: &mut Transaction<'_, Self::Db>,
        key: Self::QueryOne,
    ) -> Result<bool, DatabaseError>;
}

#[async_trait::async_trait]
pub trait BaseDatabase<Db>
where
    Db: Database,
{
    async fn transaction<F, T, E>(pool: &Pool<Db>, f: F) -> Result<T, E>
    where
        T: Send,
        E: From<DatabaseError>,
        F: for<'a> FnOnce(
                &'a mut Transaction<'_, Db>,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>
            + Send,
    {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| E::from(DatabaseError::from(e)))?;
        let result = f(&mut tx).await?;

        tx.commit()
            .await
            .map_err(|e| E::from(DatabaseError::from(e)))?;
        Ok(result)
    }
}
