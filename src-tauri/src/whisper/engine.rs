pub struct WhisperEngine;

impl WhisperEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn transcribe(&self, _audio: &[f32]) -> String {
        String::new()
    }
}
