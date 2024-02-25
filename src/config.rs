use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub label_studio_url: String,
    pub label_studio_token: String,
    pub ml_backend_addr: String,
    pub ml_backend_port: u16,
    pub ml_provider: String,
    pub ml_labels: Vec<String>,
}
