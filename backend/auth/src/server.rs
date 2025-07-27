use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::Pool;

#[derive(Debug)]
pub enum ServerError {
    JsonRejection(JsonRejection),
    InternalServerError(String),
    Unauthorized,
    BadRequest(String),
}

impl From<sqlx::Error> for ServerError {
    fn from(value: sqlx::Error) -> Self {
        tracing::error!("DatabaseError: {:?}", value);
        ServerError::InternalServerError("Internal Server Error".to_string())
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message) = match self {
            ServerError::JsonRejection(rejection) => {
                tracing::error!("Invalid Request: {:?}", rejection);
                (StatusCode::BAD_REQUEST, rejection.body_text())
            }
            ServerError::InternalServerError(e) => {
                tracing::error!("Internal Server Error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
            ServerError::Unauthorized => (StatusCode::UNAUTHORIZED, "Invalid Access".to_string()),
            ServerError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

pub struct AppState<Db>
where
    Db: sqlx::Database,
{
    pub pool: Pool<Db>,
}

impl<Db> AppState<Db>
where
    Db: sqlx::Database,
{
    pub fn new(pool: Pool<Db>) -> Self {
        Self { pool }
    }
}
