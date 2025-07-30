use auth_database::entities::{credentials::CredentialsDAO, sessions::SessionsDAO};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateCredentialDTO {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDTO {
    pub id: String,
    pub email: String,
    pub password: String,
    pub active: bool,
}

impl From<CredentialsDAO> for CredentialDTO {
    fn from(value: CredentialsDAO) -> Self {
        Self {
            id: value.id.to_string(),
            email: value.email,
            password: value.password,
            active: value.active,
        }
    }
}

impl IntoResponse for CredentialDTO {
    fn into_response(self) -> axum::response::Response {
        axum::Json::from(self).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionsDTO {
    pub id: String,
    pub credential_id: String,
    pub expires_at: String,
    pub created_at: String,
    pub active: bool,
}

impl From<SessionsDAO> for SessionsDTO {
    fn from(value: SessionsDAO) -> Self {
        Self {
            id: value.id.to_string(),
            credential_id: value.credential_id.to_string(),
            expires_at: value.expires_at.to_string(),
            created_at: value.created_at.to_string(),
            active: value.active,
        }
    }
}
