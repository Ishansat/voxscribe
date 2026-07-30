use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

use crate::audio::capture::{AudioCapture, AudioCaptureConfig};
use crate::audio::devices::{self, AudioDeviceInfo};
use crate::audio::pipeline::AudioPipeline;
use crate::detector::meeting;

const TARGET_SAMPLE_RATE: u32 = 16_000;

fn find_device_by_name(name: &str) -> Option<AudioDeviceInfo> {
    devices::enum_input_devices()
        .into_iter()
        .find(|d| d.name.eq_ignore_ascii_case(name) || d.name.contains(name))
}

pub struct SessionState {
    pub running: Arc<AtomicBool>,
    pub detector_flag: Arc<AtomicBool>,
    pub pipeline_handle: tokio::task::JoinHandle<()>,
    pub forwarder_handle: tokio::task::JoinHandle<()>,
}

pub struct SessionManager {
    pub current: Mutex<Option<SessionState>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SessionConfig {
    pub mic_device: String,
    pub system_device: String,
    pub source_language: String,
    pub target_language: String,
}

#[derive(Serialize)]
pub struct SessionInfo {
    pub source_language: String,
    pub target_language: String,
    pub mic_device: String,
    pub system_device: String,
}

#[tauri::command]
pub async fn start_voxscribe_session(
    app: AppHandle,
    config: SessionConfig,
) -> Result<SessionInfo, String> {
    let manager = app.state::<SessionManager>();
    if manager.current.lock().unwrap().is_some() {
        return Err("A session is already running".into());
    }

    let mic = find_device_by_name(&config.mic_device)
        .or_else(devices::default_microphone)
        .ok_or_else(|| {
            format!(
                "Microphone '{}' not found and no default available",
                config.mic_device
            )
        })?;

    let loopback = if config.system_device.is_empty() {
        None
    } else {
        find_device_by_name(&config.system_device).or_else(devices::default_loopback)
    };

    let capture_cfg = AudioCaptureConfig {
        target_sample_rate: TARGET_SAMPLE_RATE,
        ..Default::default()
    };

    let capture = AudioCapture::with_config(mic.clone(), loopback.clone(), capture_cfg);
    let mut audio_stream =
        capture.start().map_err(|e| format!("Audio capture failed: {}", e))?;

    let sample_rate = audio_stream.sample_rate();
    let (pipeline_tx, pipeline_rx) = mpsc::channel::<Vec<f32>>(64);
    let (detector_tx, detector_rx) = mpsc::channel::<Vec<f32>>(64);

    let detector_flag = meeting::start_detector(app.clone(), detector_rx);

    let pipeline = AudioPipeline::new(
        app.clone(),
        config.source_language.clone(),
        config.target_language.clone(),
    );

    let pipeline_handle = tokio::spawn(async move {
        pipeline.run_with_receiver(pipeline_rx, sample_rate).await;
    });

    let running = Arc::new(AtomicBool::new(true));
    let running_fwd = running.clone();

    let forwarder_handle = tokio::spawn(async move {
        let rx = audio_stream.receiver();
        while let Some(chunk) = rx.recv().await {
            if !running_fwd.load(Ordering::Relaxed) {
                break;
            }
            let _ = pipeline_tx.try_send(chunk.clone());
            let _ = detector_tx.try_send(chunk);
        }
    });

    let state = SessionState {
        running,
        detector_flag,
        pipeline_handle,
        forwarder_handle,
    };

    *manager.current.lock().unwrap() = Some(state);

    let mic_name = mic.name.clone();
    let loopback_name = loopback.as_ref().map(|d| d.name.clone());

    Ok(SessionInfo {
        source_language: config.source_language,
        target_language: config.target_language,
        mic_device: mic_name,
        system_device: loopback_name.unwrap_or_default(),
    })
}

#[tauri::command]
pub async fn stop_voxscribe_session(app: AppHandle) -> Result<String, String> {
    let manager = app.state::<SessionManager>();
    let mut guard = manager.current.lock().unwrap();

    match guard.take() {
        Some(state) => {
            state.running.store(false, Ordering::Relaxed);
            state.detector_flag.store(false, Ordering::Relaxed);
            state.pipeline_handle.abort();
            state.forwarder_handle.abort();
            drop(state);
            let _ = app.emit("session-stopped", ());
            Ok("Session stopped".into())
        }
        None => Err("No active session".into()),
    }
}
