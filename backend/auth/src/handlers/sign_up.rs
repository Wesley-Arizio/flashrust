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
