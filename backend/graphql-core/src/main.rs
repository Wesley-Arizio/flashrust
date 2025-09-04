use actix_web::{App, HttpServer};
use dotenvy::dotenv;

use actix_web::{HttpResponse, Responder, get};

use clap::Parser;

#[get("/health-check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Server address
    #[arg(long, env = "GRAPHQL_API_PORT")]
    port: u16,

    /// Database URL
    #[arg(long, env = "GRAPHQL_API_DATABASE_URL")]
    database_url: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let args = Args::parse();

    HttpServer::new(|| App::new().service(health_check))
        .bind(("127.0.0.1", args.port))?
        .run()
        .await
}
