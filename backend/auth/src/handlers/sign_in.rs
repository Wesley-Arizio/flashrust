use std::sync::Arc;
use std::time::Duration;

use auth_database::entities::sessions::CreateSessionsDAO;
use auth_database::{AuthDatabase, CredentialsRepository, SessionsRepository};
use auth_database::{
    entities::credentials::CredentialsBy,
    traits::{BaseDatabase, EntityRepository},
};
use axum::body::Body;
use axum::http::header::SET_COOKIE;
use axum::http::{Response, StatusCode};
use axum::{Json, extract::State};
use cookie::Cookie;
use cookie::time::OffsetDateTime;
use sqlx::types::chrono::{DateTime, Utc};

use crate::common::{MIN_LEN_PASSOWRD, SESSION_KEY, verify_password};
use crate::handlers::dto::SignInDTO;
use crate::{
    common::is_valid_email,
    server::{AppState, ServerError},
};

const ONE_DAY_IN_SECONDS: u64 = 60 * 60 * 24;

pub trait ChronoToTime {
    fn to_offset_datetime(&self) -> Result<OffsetDateTime, ServerError>;
}

impl ChronoToTime for DateTime<Utc> {
    fn to_offset_datetime(&self) -> Result<OffsetDateTime, ServerError> {
        OffsetDateTime::from_unix_timestamp(self.timestamp())
            .and_then(|dt| dt.replace_nanosecond(self.timestamp_subsec_nanos()))
            .map_err(|e| ServerError::InternalServerError(e.to_string()))
    }
}

pub async fn sign_in<DB>(
    State(state): State<Arc<AppState<DB>>>,
    Json(payload): Json<SignInDTO>,
) -> Result<Response<Body>, ServerError>
where
    DB: sqlx::Database,
    CredentialsRepository: EntityRepository<Db = DB>,
    SessionsRepository: EntityRepository<Db = DB>,
{
    if !is_valid_email(&payload.email)? {
        return Err(ServerError::BadRequest("Invalid Email Format".to_string()));
    };

    if payload.password.len() < MIN_LEN_PASSOWRD {
        return Err(ServerError::BadRequest(format!(
            "Password must be at least {MIN_LEN_PASSOWRD} characters long",
        )));
    }

    AuthDatabase::transaction(&state.pool, |tx| {
        Box::pin(async move {
            let maybe_credential =
                CredentialsRepository::try_get(tx, CredentialsBy::Email(payload.email.clone()))
                    .await?;

            let Some(credential) = maybe_credential else {
                return Err(ServerError::Unauthorized);
            };

            if !credential.active {
                return Err(ServerError::Unauthorized);
            };

            let is_correct_password = verify_password(&payload.password, &credential.password)?;

            if !is_correct_password {
                return Err(ServerError::Unauthorized);
            };

            let session = CreateSessionsDAO {
                credential_id: credential.id,
                expires_at: Utc::now() + Duration::from_secs(ONE_DAY_IN_SECONDS),
            };

            let session = SessionsRepository::insert(tx, session)
                .await
                .map_err(ServerError::from)?;

            let id = session.id.to_string();
            let cookie = cookie(&id, session.expires_at.to_offset_datetime()?);

            let response = Response::builder()
                .status(StatusCode::OK)
                .header(SET_COOKIE, cookie.to_string())
                .body(Body::empty())
                .map_err(|e| {
                    tracing::error!("Error building request: {:#?}", e);
                    ServerError::InternalServerError("Internal Server Error".to_string())
                })?;

            Ok(response)
        })
    })
    .await
}

fn cookie(value: &str, expires_at: OffsetDateTime) -> Cookie<'_> {
    Cookie::build((SESSION_KEY, value))
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(expires_at)
        .build()
}

#[cfg(any(feature = "unit", feature = "integration"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::App;
    use auth_database::{
        AuthDatabase, CredentialsRepository,
        entities::credentials::{CreateCredentialsDAO, CredentialsBy},
        traits::{BaseDatabase, EntityRepository},
    };
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use serde_json::Value;

    use sqlx::Pool;
    use tower::Service;
    use tower::util::ServiceExt;

    #[cfg(feature = "unit")]
    use sqlx::Sqlite;

    #[cfg(feature = "integration")]
    use sqlx::Postgres;

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
    async fn sign_in_invalid_email() {
        let (_, app) = setup().await;
        let mut app = app.into_service();
        let body = serde_json::json!({
            "email": "owkmail.com",
            "password": "ondfauhdf77364"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/sign_in")
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
    async fn sign_in_short_password() {
        let (_, app) = setup().await;
        let mut app = app.into_service();
        let body = serde_json::json!({
            "email": "test@gmail.com",
            "password": "EYMQE"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/sign_in")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.ready().await.unwrap().call(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(
            json.get("message").unwrap(),
            "Password must be at least 6 characters long"
        );
    }

    #[tokio::test]
    async fn sign_in_non_existent_credential() {
        let (_, app) = setup().await;
        let mut app = app.into_service();
        let body = serde_json::json!({
            "email": "test@gmail.com",
            "password": "Ej42fkj!yI!Cj9"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/sign_in")
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
    async fn sign_in_deactivated_account() {
        let (pool, app) = setup().await;

        AuthDatabase::transaction(&pool, |tx| {
            Box::pin(async move {
                let credential = CreateCredentialsDAO {
                    email: "test@gmail.com".to_string(),
                    password: "Ej42fkj!yI!Cj9".to_string(),
                };
                let credential = CredentialsRepository::insert(tx, credential).await.unwrap();

                // Deactivate Account
                let _ = CredentialsRepository::delete(tx, CredentialsBy::Id(credential.id))
                    .await
                    .unwrap();

                Ok::<(), auth_database::traits::DatabaseError>(())
            })
        })
        .await
        .expect("Could not setup deactived account for this test");

        let mut app = app.into_service();
        let body = serde_json::json!({
            "email": "test@gmail.com",
            "password": "Ej42fkj!yI!Cj9"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/sign_in")
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
    async fn sign_in_incorrect_password() {
        let (_, app) = setup().await;

        let mut app = app.into_service();
        let body = serde_json::json!({
            "email": "test22@gmail.com",
            "password": "Ej24a2fkj!yI!Cj9"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/sign_up")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = serde_json::json!({
            "email": "test22@gmail.com",
            "password": "!yI!Cj9"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/sign_in")
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
    async fn sign_in_success() {
        let (_, app) = setup().await;

        let mut app = app.into_service();
        let body = serde_json::json!({
            "email": "test2@gmail.com",
            "password": "Ej4a2fkj!yI!Cj9"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/sign_up")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let request = Request::builder()
            .method("POST")
            .uri("/sign_in")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.ready().await.unwrap().call(request).await.unwrap();

        let cookie_header = response
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap();

        let cookie = Cookie::parse(cookie_header).unwrap();
        assert_eq!(cookie.name(), SESSION_KEY);
        assert_eq!(cookie.path(), Some("/"));
        assert!(cookie.secure().unwrap());
        assert!(cookie.http_only().unwrap());
        let expected = Utc::now() + Duration::from_secs(ONE_DAY_IN_SECONDS);
        let expires = cookie
            .expires_datetime()
            .expect("cookie must have an expiration");
        let diff = (expires - expected.to_offset_datetime().unwrap())
            .whole_seconds()
            .abs();
        assert!(diff <= 1, "Max-Age is not ~24h");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
