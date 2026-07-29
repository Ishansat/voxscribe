use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Serialize)]
pub struct TranscriptBlockPayload {
    pub id: String,
    pub transcribed_text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub timestamp: String,
}

impl TranscriptBlockPayload {
    pub fn new(
        transcribed_text: String,
        translated_text: String,
        source_lang: String,
        target_lang: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            transcribed_text,
            translated_text,
            source_lang,
            target_lang,
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}
