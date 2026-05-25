use anyhow::Result;
use cli_memory_engine::{Storage, VectorStore, current_model_dir, model_cache_ready};
use cli_memory_integrations::detect_providers;
use serde::Serialize;

use crate::bootstrap::{configured_data_dir, configured_db_path, configured_home};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DoctorStatus {
    pub db_ready: bool,
    pub embedder_ready: bool,
    pub vector_ready: bool,
    pub detected_providers: Vec<String>,
    pub data_dir: String,
    pub db_path: String,
    pub active_conversations: i64,
    pub total_conversations: i64,
    pub total_messages: i64,
    pub total_embeddings: i64,
    pub total_checkpoints: i64,
    pub model_dir: String,
}

pub fn inspect() -> Result<DoctorStatus> {
    let home = configured_home()?;
    let data_dir = configured_data_dir()?;
    let db_path = configured_db_path()?;

    let detected_providers = detect_providers(&home)?
        .into_iter()
        .map(|item| item.provider.as_slug().to_owned())
        .collect::<Vec<_>>();

    let storage = Storage::open(&db_path)?;
    storage.initialize()?;

    let model_dir = current_model_dir()?;
    let embedder_ready = model_cache_ready()?;
    let vector = VectorStore::new(128)?;

    Ok(DoctorStatus {
        db_ready: true,
        embedder_ready,
        vector_ready: vector.search(&vec![0.0; 128], 1).is_ok(),
        detected_providers,
        data_dir: data_dir.display().to_string(),
        db_path: db_path.display().to_string(),
        active_conversations: storage.count_active_conversations()?,
        total_conversations: storage.count_conversations()?,
        total_messages: storage.count_messages()?,
        total_embeddings: storage.count_message_embeddings()?,
        total_checkpoints: i64::try_from(storage.list_checkpoints()?.len())
            .expect("checkpoint count should fit in i64"),
        model_dir: model_dir.display().to_string(),
    })
}
