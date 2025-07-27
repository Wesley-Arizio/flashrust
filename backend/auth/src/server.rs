use std::{marker::PhantomData, sync::Arc};

use axum::{
    Json, Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Serialize;
use sqlx::{Pool, Postgres};

use crate::database::traits::CredentialsEntityRepository;

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
        println!("DatabaseError: {:?}", value);
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
            ServerError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ServerError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

#[derive(Clone)]
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

pub struct App<R>
where
    R: CredentialsEntityRepository + Send + Sync + 'static,
    R::Error: Into<ServerError>,
{
    phantom_data: PhantomData<R>,
}

impl<R> App<R>
where
    R: CredentialsEntityRepository + Send + Sync + 'static,
    R::Error: Into<ServerError>,
{
    pub fn new(pool: Pool<R::Db>) -> Router {
        let app_state = Arc::new(AppState::new(pool));

        Router::new()
            .route("/sign_up", post(crate::handlers::sign_up::sign_up::<R>))
            .with_state(app_state)
    }

    #[cfg(feature = "default")]
    pub async fn run(pool: Pool<Postgres>, address: &str) {
        let app = App::<crate::database::postgres::PostgresCredentialsRepository>::new(pool);

        match tokio::net::TcpListener::bind(&address).await {
            Ok(listener) => {
                tracing::info!("Auth server running at https://{}", address);
                if let Err(e) = axum::serve(listener, app).await {
                    tracing::error!("Error starting auth microservice: {:?}", e);
                }
            }
            Err(e) => {
                tracing::error!("Error binding server to the address: {:?}", e);
            }
        };
    }
}
