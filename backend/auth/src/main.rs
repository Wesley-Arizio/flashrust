use dotenvy::dotenv;

use clap::{Parser, command};
use sqlx::PgPool;

use crate::{database::CredentialsRepository, server::App};

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

    App::<CredentialsRepository>::run(pool, &args.address).await;
}
