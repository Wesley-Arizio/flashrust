use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::database::traits::CredentialDAO;

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

impl From<CredentialDAO> for CredentialDTO {
    fn from(value: CredentialDAO) -> Self {
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
