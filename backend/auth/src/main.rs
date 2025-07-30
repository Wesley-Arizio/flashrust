use dotenvy::dotenv;

use clap::{Parser, command};

use crate::server::App;

pub mod common;
pub mod handlers;
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

    App::run(&args.database_url, &args.address).await;
}
