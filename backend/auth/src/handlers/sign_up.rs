use std::sync::Arc;

use auth_database::{AuthDatabase, CredentialsRepository};
use auth_database::{
    entities::credentials::{CreateCredentialsDAO, CredentialsBy},
    traits::{BaseDatabase, EntityRepository},
};
use axum::{Json, extract::State};

use crate::{
    common::{hash_password, is_valid_email, is_valid_password},
    handlers::dto::{CreateCredentialDTO, CredentialsDTO},
    server::{AppState, ServerError},
};

pub async fn sign_up<DB>(
    State(state): State<Arc<AppState<DB>>>,
    Json(payload): Json<CreateCredentialDTO>,
) -> Result<CredentialsDTO, ServerError>
where
    DB: sqlx::Database,
    CredentialsRepository: EntityRepository<Db = DB>,
{
    if !is_valid_email(&payload.email)? {
        return Err(ServerError::BadRequest("Invalid Email Format".to_string()));
    };

    if !is_valid_password(&payload.password) {
        return Err(ServerError::BadRequest(
            "Invalid Password Format".to_string(),
        ));
    }

    AuthDatabase::transaction(&state.pool, |tx| {
        Box::pin(async move {
            let exists =
                CredentialsRepository::exists(tx, CredentialsBy::Email(payload.email.clone()))
                    .await?;

            if exists {
                return Err(ServerError::Unauthorized);
            };

            let hash = hash_password(&payload.password)?;
            let credential_dao = CreateCredentialsDAO {
                email: payload.email,
                password: hash,
            };

            let create_credential = CredentialsRepository::insert(tx, credential_dao)
                .await
                .map_err(ServerError::from)?;

            Ok(CredentialsDTO::from(create_credential))
        })
    })
    .await
}

#[cfg(any(feature = "unit", feature = "integration"))]
#[cfg(test)]
mod tests {
    use crate::server::App;

    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use serde_json::Value;
    use tower::Service;
    use tower::util::ServiceExt;

    use sqlx::Pool;

    #[cfg(feature = "unit")]
    use sqlx::Sqlite;

    #[cfg(feature = "integration")]
    use sqlx::Postgres;

    use auth_database::AuthDatabase;

    #[cfg(feature = "unit")]
    async fn setup() -> (Pool<Sqlite>, Router) {
        let pool = AuthDatabase::connect(":memory:").await.unwrap();
        (pool.clone(), App::app(pool).await)
    }

    #[cfg(feature = "integration")]
    async fn setup() -> (Pool<Postgres>, Router) {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("AUTH_DATABASE_URL")
            .expect("AUTH_DATABASE_URL must be set for integration tests");

        let pool = AuthDatabase::connect(&database_url).await.unwrap();
        (pool.clone(), App::app(pool).await)
    }

    #[tokio::test]
    async fn signup_invalid_email() {
        let (_, app) = setup().await;
        let mut app = app.into_service();
        let body = serde_json::json!({
            "email": "owkmail.com",
            "password": "ondfauhdf77364"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/sign_up")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.ready().await.unwrap().call(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(json.get("message").unwrap(), "Invalid Email Format");
    }

    #[tokio::test]
    async fn signup_invalid_password() {
        let (_, app) = setup().await;
        let mut app = app.into_service();
        let body = serde_json::json!({
            "email": "owk@mail.com",
            "password": "444"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/sign_up")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.ready().await.unwrap().call(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(json.get("message").unwrap(), "Invalid Password Format");
    }

    #[tokio::test]
    async fn sign_up_credentials_already_exists() {
        let (_, app) = setup().await;

        let body = serde_json::json!({
            "email": "owkw22222@mail.com",
            "password": "asdjfnaksdf87"
        });

        let mut app = app.into_service();
        let request = Request::builder()
            .method("POST")
            .uri("/sign_up")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = serde_json::json!({
            "email": "owkw22222@mail.com",
            "password": "asdjfnaksdf87"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/sign_up")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.ready().await.unwrap().call(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(parts.status, StatusCode::UNAUTHORIZED);
        assert_eq!(json.get("message").unwrap(), "Unauthorized");
    }

    #[tokio::test]
    async fn signup_success() {
        let (_, app) = setup().await;

        let mut app = app.into_service();
        let body = serde_json::json!({
            "email": "asdfasdfasdf@mail.com",
            "password": "asdjfnaksdf87"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/sign_up")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.ready().await.unwrap().call(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(json.get("email").unwrap(), "asdfasdfasdf@mail.com");
        assert_eq!(json.get("active").unwrap(), true);

        let hash = json.get("password").unwrap().as_str().unwrap();
        let parsed_hash = PasswordHash::new(&hash).unwrap();

        assert!(
            Argon2::default()
                .verify_password(b"asdjfnaksdf87", &parsed_hash)
                .is_ok()
        );
    }
}
