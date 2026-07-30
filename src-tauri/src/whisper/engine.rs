use std::path::Path;

use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

use crate::whisper::model_loader::ModelLoader;

const MODEL_FILENAME: &str = "ggml-large-v3-turbo-q8_0.bin";

pub struct WhisperEngine {
    ctx: WhisperContext,
}

impl WhisperEngine {
    pub fn new() -> Self {
        let loader = ModelLoader::new();

        let model_path = Path::new(MODEL_FILENAME);
        let model_path = if model_path.exists() {
            model_path.to_path_buf()
        } else {
            loader
                .find_model(MODEL_FILENAME)
                .unwrap_or_else(|| model_path.to_path_buf())
        };

        let ctx = WhisperContext::new_with_params(
            model_path.to_string_lossy().as_ref(),
            WhisperContextParameters::default(),
        )
        .expect("failed to load Whisper Large v3 Turbo model");

        Self { ctx }
    }

    pub fn from_path<P: AsRef<Path>>(model_path: P) -> Result<Self, String> {
        let ctx = WhisperContext::new_with_params(
            model_path.as_ref().to_string_lossy().as_ref(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| format!("failed to load model: {}", e))?;

        Ok(Self { ctx })
    }

    pub fn transcribe(&self, pcm_data: &[f32]) -> Result<String, String> {
        let n_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_n_threads(n_threads as i32);
        params.set_no_context(true);
        params.set_single_segment(true);

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| format!("failed to create state: {}", e))?;

        state
            .full(params, pcm_data)
            .map_err(|e| format!("transcription failed: {}", e))?;

        collect_text(&state)
    }
}

fn collect_text(state: &WhisperState) -> Result<String, String> {
    let num_segments = state
        .full_n_segments()
        .map_err(|e| format!("failed to get segment count: {}", e))?;

    let mut result = String::with_capacity(512);

    for i in 0..num_segments {
        let text = state
            .full_get_segment_text(i)
            .map_err(|e| format!("failed to get segment {}: {}", i, e))?;
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(text.trim());
    }

    Ok(result)
}
