use anyhow::Result;
use trad::Translator;

pub struct LocalTranslationEngine {
    translator: Translator,
}

impl LocalTranslationEngine {
    pub fn new() -> Self {
        let rt = tokio::runtime::Handle::try_current()
            .expect("LocalTranslationEngine::new() must be called from a Tokio context");

        let translator = rt
            .block_on(Translator::setup(None))
            .expect("failed to initialize local translation engine");

        Self { translator }
    }

    pub async fn new_async() -> Result<Self> {
        let translator = Translator::setup(None).await?;
        Ok(Self { translator })
    }

    pub async fn translate(
        &self,
        text: &str,
        src_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }

        let translated = self.translator.translate(text, src_lang, target_lang).await?;
        Ok(translated)
    }
}
