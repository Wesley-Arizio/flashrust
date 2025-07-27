use std::sync::Arc;

use axum::{Router, routing::post};

use dotenvy::dotenv;

use clap::{Parser, command};
use sqlx::PgPool;

use crate::{database::CredentialsRepository, server::AppState};

pub mod handlers;

pub mod database;
pub mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Server address
    #[arg(long, env = "AUTH_SERVER_ADDRESS")]
    address: String,

    #[arg(long, env = "AUTH_DATABASE_URL")]
    database_url: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let args = Args::parse();

    let pool = PgPool::connect(&args.database_url)
        .await
        .expect("Could not connect with database");

    let app_state = Arc::new(AppState::new(pool));

    let app = Router::new()
        .route(
            "/sign_up",
            post(crate::handlers::sign_up::sign_up::<CredentialsRepository>),
        )
        .with_state(app_state);

    match tokio::net::TcpListener::bind(&args.address).await {
        Ok(listener) => {
            tracing::info!("Auth server running at https://{}", args.address);
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("Error starting auth microservice: {:?}", e);
            }
        }
        Err(e) => {
            tracing::error!("Error binding server to the address: {:?}", e);
        }
    };
}
