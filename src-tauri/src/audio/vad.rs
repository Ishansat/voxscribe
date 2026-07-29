pub struct VoiceActivityDetector;

impl VoiceActivityDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn is_speech(&self, _audio: &[f32]) -> bool {
        false
    }
}
