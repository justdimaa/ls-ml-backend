use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SetupRequest {
    pub project: String,
    pub schema: String,
    pub hostname: String,
    pub access_token: String,
    pub model_version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SetupResponse {
    pub model_version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PredictRequest {
    pub tasks: Vec<Task>,
    pub model_version: Option<String>,
    pub project: String,
    pub label_config: String,
    pub params: Params,
}

#[derive(Debug, Serialize)]
pub struct PredictResponse {
    pub result: Vec<Pred>,
    pub score: f32,
    pub model_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub id: u32,
    pub data: TaskData,
    pub meta: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
    pub is_labeled: bool,
    pub overlap: f64,
    pub inner_id: u32,
    pub total_annotations: u32,
    pub cancelled_annotations: u32,
    pub total_predictions: u32,
    pub comment_count: u32,
    pub unresolved_comment_count: u32,
    pub last_comment_updated_at: Option<String>,
    pub project: u32,
    pub updated_by: Option<u32>,
    pub file_upload: u32,
    pub comment_authors: Vec<String>,
    pub annotations: Vec<serde_json::Value>,
    pub predictions: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskData {
    pub image: String,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    pub login: Option<String>,
    pub password: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Prediction {
    pub id: u32,
    pub model_version: String,
    pub created_ago: String,
    pub result: Vec<serde_json::Value>,
    pub score: Option<f64>,
    pub cluster: Option<String>,
    pub neighbors: Option<Vec<String>>,
    pub mislabeling: f64,
    pub created_at: String,
    pub updated_at: String,
    pub task: u32,
    pub project: u32,
}

#[derive(Debug, Serialize)]
pub struct Pred {
    pub id: String,
    pub from_name: String,
    pub to_name: String,
    #[serde(rename = "type")]
    pub t: String,
    pub score: f32,
    pub original_width: u32,
    pub original_height: u32,
    pub image_rotation: u32,
    pub value: PredValue,
    pub readonly: bool,
}

#[derive(Debug, Serialize)]
pub struct PredValue {
    pub rotation: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rectanglelabels: Vec<String>,
}

pub fn get_project_id(project: &String) -> Option<u32> {
    project.split(".").next()?.parse::<u32>().ok()
}
