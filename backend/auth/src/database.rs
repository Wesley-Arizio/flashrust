pub mod postgres;
pub mod sqlite;
pub mod traits;

#[cfg(feature = "unit")]
pub use sqlite::SqliteCredentialsRepository as CredentialsRepository;

#[cfg(not(feature = "unit"))]
pub use postgres::PostgresCredentialsRepository as CredentialsRepository;
