use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

#[derive(Debug)]
pub struct AppState {
    pub models: RwLock<HashMap<u32, Arc<ort::Session>>>,
}
