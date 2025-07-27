use std::sync::Arc;

use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State};
use regex::Regex;

use crate::{
    database::traits::{CreateCredentialDAO, CredentialsEntityRepository},
    handlers::dto::{CreateCredentialDTO, CredentialDTO},
    server::{AppState, ServerError},
};

const MIN_LEN_PASSOWRD: usize = 6;

pub async fn sign_up<R>(
    State(state): State<Arc<AppState<R::Db>>>,
    Json(payload): Json<CreateCredentialDTO>,
) -> Result<CredentialDTO, ServerError>
where
    R: CredentialsEntityRepository,
    R::Error: Into<ServerError>,
{
    if !is_valid_email(&payload.email)? {
        return Err(ServerError::BadRequest("Invalid Email Format".to_string()));
    };

    if !is_valid_password(&payload.password) {
        return Err(ServerError::BadRequest(
            "Invalid Password Format".to_string(),
        ));
    }

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| ServerError::InternalServerError(e.to_string()))?;

    let exists = R::exists(&mut tx, &payload.email)
        .await
        .map_err(|e| e.into())?;

    if exists {
        return Err(ServerError::Unauthorized);
    };

    let hash = hash_password(&payload.password)?;
    let credential_dao = CreateCredentialDAO {
        email: payload.email,
        password: hash,
    };

    let create_credential = R::create(&mut tx, credential_dao)
        .await
        .map_err(|e| e.into())?;

    tx.commit()
        .await
        .map_err(|e| ServerError::InternalServerError(e.to_string()))?;

    Ok(create_credential.into())
}

fn is_valid_password(password: &str) -> bool {
    password.len() >= MIN_LEN_PASSOWRD
}

fn is_valid_email(email: &str) -> Result<bool, ServerError> {
    let regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|e| ServerError::InternalServerError(e.to_string()))?;

    Ok(regex.is_match(email))
}

fn hash_password(password: &str) -> Result<String, ServerError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ServerError::InternalServerError(e.to_string()))?
        .to_string())
}

#[cfg(any(feature = "unit", feature = "integration"))]
#[cfg(test)]
mod tests {
    use crate::{
        database::CredentialsRepository,
        database::traits::{CreateCredentialDAO, CredentialsEntityRepository},
        handlers::sign_up::{is_valid_email, is_valid_password},
        server::App,
    };

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

    #[cfg(feature = "unit")]
    async fn setup() -> (Router, sqlx::Pool<sqlx::Sqlite>) {
        let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();

        sqlx::migrate!("./sqlite")
            .run(&pool)
            .await
            .expect("Could not run migration on sqlite database");

        let app = App::<CredentialsRepository>::new(pool.clone());
        (app, pool)
    }

    #[cfg(feature = "integration")]
    async fn setup() -> (Router, sqlx::Pool<sqlx::Postgres>) {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("AUTH_DATABASE_URL")
            .expect("AUTH_DATABASE_URL must be set for integration tests");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Could not connect with database");
        let app = App::<CredentialsRepository>::new(pool.clone());
        (app, pool)
    }

    #[test]
    fn valid_emails() {
        let emails = [
            "simple@example.com",
            "very.common@example.com",
            "disposable.style.email.with+symbol@example.com",
            "other.email-with-hyphen@example.com",
            "fully-qualified-domain@example.co.uk",
            "user.name+tag+sorting@example.com",
            "x@example.com",
            "example-indeed@strange-example.com",
            "user%example.com@example.org",
            "user_name@example.org",
            "user123@sub.domain.example.com",
            "example@s.solution",
            "a@b.co",
        ];

        for email in emails.iter() {
            assert!(
                is_valid_email(email).unwrap(),
                "failed for email: {:?}",
                email
            );
        }
    }

    #[test]
    fn invalid_emails() {
        let emails = [
            "admin@mailserver1",
            "\"john..doe\"@example.org",
            "\"much.more unusual\"@example.com",
            "\"very.unusual.@.unusual.com\"@example.com",
            "\"very.(),:;<>[]\".VERY.\"very@\\ \"very\".unusual\"@strange.example.com",
            "test@xn--d1acufc.xn--p1ai",
            "test@пример.рф",
        ];

        for email in emails.iter() {
            assert!(
                !is_valid_email(email).unwrap(),
                "failed for email: {:?}",
                email
            );
        }
    }

    #[test]
    fn valid_password() {
        let password = "anaksfdb3434bbc";
        assert!(is_valid_password(password));
    }

    #[test]
    fn invalid_password() {
        let password = "anak3";
        assert!(!is_valid_password(password));
    }

    #[tokio::test]
    async fn signup_invalid_email() {
        let (app, _) = setup().await;
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
        let (app, _) = setup().await;
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
    async fn signup_invalid_credentials() {
        let (app, pool) = setup().await;

        let credential = CreateCredentialDAO {
            email: "owkw22222@mail.com".to_string(),
            password: "test".to_string(),
        };

        let mut tx = pool.begin().await.unwrap();

        CredentialsRepository::create(&mut tx, credential)
            .await
            .expect("Could not create credential");

        tx.commit().await.unwrap();

        let mut app = app.into_service();
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
        let (app, _) = setup().await;

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
