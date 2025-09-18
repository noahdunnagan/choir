use crate::config::EnvConfig;
use crate::modules::{choir::ChoirService, openai};
use crate::routes::configure_routes;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod ai_functions;
mod config;
mod macros;
mod modules;
mod response;
mod routes;
mod types;
mod utils;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = Arc::new(EnvConfig::from_env());
    let addr = format!("0.0.0.0:{}", config.port);

    println!("Starting server on {}", addr);
    let openai_service = Arc::new(openai::OpenAIService::new(config.clone()).await);
    let choir_service = web::Data::new(ChoirService::new(openai_service.clone()));

    HttpServer::new(move || {
        App::new()
            .configure(configure_routes)
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::from(openai_service.clone()))
            .app_data(choir_service.clone())
    })
    .bind(addr)?
    .run()
    .await
}
