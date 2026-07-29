use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Serialize, Deserialize)]
pub struct SessionConfig {
    pub mic_device: String,
    pub system_device: String,
    pub source_language: String,
    pub target_language: String,
}

#[tauri::command]
pub async fn start_voxscribe_session(
    _app: AppHandle,
    config: SessionConfig,
) -> Result<String, String> {
    Ok(format!(
        "Session started: {} -> {} (mic: {}, system: {})",
        config.source_language, config.target_language, config.mic_device, config.system_device
    ))
}

#[tauri::command]
pub async fn stop_voxscribe_session() -> Result<String, String> {
    Ok("Session stopped".into())
}
