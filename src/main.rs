use std::{collections::HashMap, error::Error};

use actix_web::{middleware::Logger, web, App, HttpServer};
use ort::ExecutionProvider;
use state::AppState;
use tokio::sync::RwLock;

mod api;
mod config;
mod services;
mod state;

const MODEL_VERSION: &str = "1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cfg = envy::from_env::<config::Config>().unwrap();

    let cfg_data = web::Data::new(cfg.clone());
    let state_data = web::Data::new(AppState {
        models: RwLock::new(HashMap::new()),
    });

    let ort_builder = ort::init();
    let mut ort_provider = match cfg.ml_provider.as_str() {
        "cuda" => ort::CUDAExecutionProvider::default().build(),
        _ => ort::CPUExecutionProvider::default().build(),
    };

    if !ort_provider.is_available().unwrap_or_default() {
        tracing::warn!(
            "execution provider {} unavailable, falling back to cpu",
            ort_provider.as_str()
        );
        ort_provider = ort::CPUExecutionProvider::default().build()
    }

    ort_builder
        .with_execution_providers([ort_provider])
        .commit()
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(cfg_data.clone())
            .app_data(state_data.clone())
            .service(services::health)
            .service(services::setup)
            .service(services::predict)
    })
    .bind((cfg.ml_backend_addr, cfg.ml_backend_port))?
    .run()
    .await?;

    Ok(())
}
