use sqlx::{Postgres, Transaction};

use crate::database::traits::{CreateCredentialDAO, CredentialDAO, CredentialsEntityRepository};

pub struct PostgresCredentialsRepository {}

#[async_trait::async_trait]
impl CredentialsEntityRepository for PostgresCredentialsRepository {
    type Db = Postgres;
    type Error = String;

    async fn exists(tx: &mut Transaction<'_, Self::Db>, email: &str) -> Result<bool, Self::Error> {
        let result =
            sqlx::query_as::<_, (bool,)>("SELECT EXISTS (SELECT 1 FROM credentials where email = $1)")
                .bind(email)
                .fetch_one(&mut **tx)
                .await
                .unwrap();
        Ok(result.0)
    }

    async fn create(
        tx: &mut Transaction<'_, Self::Db>,
        credential: CreateCredentialDAO,
    ) -> Result<CredentialDAO, Self::Error> {
        let result = sqlx::query_as::<_, CredentialDAO>("INSERT INTO credentials (email, password) VALUES ($1, $2) RETURNING id, email, password, active;")
            .bind(credential.email)
            .bind(credential.password)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;

        Ok(result)
    }
}
