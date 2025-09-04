use std::sync::Arc;

use actix_web::{
    App, HttpServer, middleware, route,
    web::{self, Data},
};
use dotenvy::dotenv;

use actix_web::{HttpResponse, Responder, get};

use clap::Parser;
use juniper::http::{GraphQLRequest, graphiql::graphiql_source};

mod schema;

use crate::schema::{Schema, create_schema};

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

#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    web::Html::new(graphiql_source("/graphql", None))
}

#[route("/graphql", method = "GET", method = "POST")]
async fn graphql(st: web::Data<Schema>, data: web::Json<GraphQLRequest>) -> impl Responder {
    let user = data.execute(&st, &()).await;
    HttpResponse::Ok().json(user)
}

#[get("/health-check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let args = Args::parse();

    let schema = Arc::new(create_schema());

    HttpServer::new(move || {
        App::new()
            .app_data(Data::from(schema.clone()))
            .service(graphql)
            .service(graphql_playground)
            .service(health_check)
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
