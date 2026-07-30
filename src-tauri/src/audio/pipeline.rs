use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::capture::AudioStream;
use super::vad::{SpeechSegment, VadConfig, VoiceActivityDetector};
use crate::translation::engine::LocalTranslationEngine;
use crate::whisper::engine::WhisperEngine;

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

pub struct AudioPipeline {
    app_handle: AppHandle,
    whisper: Arc<WhisperEngine>,
    translator: Arc<LocalTranslationEngine>,
    source_lang: String,
    target_lang: String,
}

impl AudioPipeline {
    pub fn new(
        app_handle: AppHandle,
        source_lang: String,
        target_lang: String,
    ) -> Self {
        Self {
            app_handle,
            whisper: Arc::new(WhisperEngine::new()),
            translator: Arc::new(LocalTranslationEngine::new()),
            source_lang,
            target_lang,
        }
    }

    pub async fn run(&self, mut audio_stream: AudioStream) {
        let vad_config = VadConfig {
            sample_rate: audio_stream.sample_rate(),
            ..Default::default()
        };
        let mut vad = VoiceActivityDetector::new(vad_config);

        while let Some(chunk) = audio_stream.receiver().recv().await {
            if let Some(segment) = vad.push_frame(&chunk) {
                self.process_segment(segment).await;
            }
        }

        if let Some(segment) = vad.flush() {
            self.process_segment(segment).await;
        }
    }

    pub async fn run_with_receiver(
        &self,
        mut rx: mpsc::Receiver<Vec<f32>>,
        sample_rate: u32,
    ) {
        let vad_config = VadConfig {
            sample_rate,
            ..Default::default()
        };
        let mut vad = VoiceActivityDetector::new(vad_config);

        while let Some(chunk) = rx.recv().await {
            if let Some(segment) = vad.push_frame(&chunk) {
                self.process_segment(segment).await;
            }
        }

        if let Some(segment) = vad.flush() {
            self.process_segment(segment).await;
        }
    }

    async fn process_segment(&self, segment: SpeechSegment) {
        let app = self.app_handle.clone();
        let whisper = self.whisper.clone();
        let translator = self.translator.clone();
        let source_lang = self.source_lang.clone();
        let target_lang = self.target_lang.clone();

        tokio::spawn(async move {
            let transcribed = match whisper.transcribe(&segment.samples) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("transcription error: {}", e);
                    return;
                }
            };

            let translated = if source_lang == target_lang {
                transcribed.clone()
            } else {
                match translator.translate(&transcribed, &source_lang, &target_lang).await {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("translation error: {}", e);
                        return;
                    }
                }
            };

            let payload = TranscriptBlockPayload::new(
                transcribed,
                translated,
                source_lang,
                target_lang,
            );

            let _ = app.emit("live-transcript-block", payload);
        });
    }
}
