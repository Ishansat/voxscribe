pub struct LocalTranslationEngine;

impl LocalTranslationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn translate(&self, text: &str, _src_lang: &str, _target_lang: &str) -> String {
        text.to_string()
    }
}
