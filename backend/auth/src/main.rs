use axum::{Router, routing::post};

use dotenvy::dotenv;

use clap::{Parser, command};

pub mod handlers;

pub mod database;
pub mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Server address
    #[arg(long, env = "AUTH_SERVER_ADDRESS")]
    address: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let args = Args::parse();

    let app = Router::new().route("/sign_up", post(crate::handlers::sign_up::sign_up));

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
