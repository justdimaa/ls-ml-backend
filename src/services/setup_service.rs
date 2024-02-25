use actix_web::{post, web, Responder};

use crate::{api, state::AppState, MODEL_VERSION};

#[post("/setup")]
async fn setup(
    req: web::Json<api::SetupRequest>,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let project_id = api::get_project_id(&req.project)
        .ok_or(actix_web::error::ErrorBadRequest("invalid project id"))?;
    let mut models = data.models.write().await;

    if models.contains_key(&project_id) {
        return Ok(web::Json(api::SetupResponse {
            model_version: Some(MODEL_VERSION.to_string()),
        }));
    }

    tracing::info!("setting up model for project id {}", project_id);

    let model = ort::Session::builder()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .with_model_from_file(format!("models/{}.onnx", project_id))
        .map_err(|e| actix_web::error::ErrorNotFound(e))?;

    models.insert(project_id, model.into());

    Ok(web::Json(api::SetupResponse {
        model_version: Some(MODEL_VERSION.to_string()),
    }))
}
