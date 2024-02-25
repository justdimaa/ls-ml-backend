use actix_web::{get, web, Responder};

use crate::api;

#[get("/health")]
pub async fn health() -> impl Responder {
    web::Json(api::HealthResponse {
        status: "UP".to_string(),
    })
}
