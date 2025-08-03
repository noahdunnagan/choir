use actix_web::{App, HttpServer, web};
use crate::config::EnvConfig;
use crate::routes::configure_routes;
use crate::modules::openai;

mod config;
mod routes;
mod response;
mod utils;
mod macros;
mod modules;
mod types;
mod ai_functions;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = EnvConfig::from_env();
    let addr = format!("0.0.0.0:{}", config.port);

    println!("Starting server on {}", addr);
    let openai_service = web::Data::new(openai::OpenAIService::new().await);

    HttpServer::new(move || {
        App::new()
            .configure(configure_routes)
            .app_data(openai_service.clone())
    })
    .bind(addr)?
    .run()
    .await
}

