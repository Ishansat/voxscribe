use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use sysinfo::System;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio::time::sleep;

const MEETING_PROCESS_NAMES: &[&str] = &[
    "zoom", "teams", "slack", "webex", "chrome",
];

const RMS_THRESHOLD: f32 = 0.05;
const PROCESS_CHECK_INTERVAL: Duration = Duration::from_secs(3);
const SIDEBAR_COOLDOWN: Duration = Duration::from_secs(30);

fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

fn is_meeting_running(system: &mut System) -> bool {
    system.refresh_processes();
    for process in system.processes().values() {
        let name = process.name();
        let lower = name.to_lowercase();
        if MEETING_PROCESS_NAMES.iter().any(|kw| lower.contains(kw)) {
            return true;
        }
    }
    false
}

pub fn start_detector(
    app_handle: AppHandle,
    audio_rx: mpsc::Receiver<Vec<f32>>,
) -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();

    tokio::spawn(async move {
        let mut system = System::new();
        let mut sidebar_open = false;
        let mut recent_rms: Vec<f32> = Vec::with_capacity(16);
        let mut audio_rx = audio_rx;

        while running.load(Ordering::Relaxed) {
            let meeting_detected = is_meeting_running(&mut system);

            while let Ok(chunk) = audio_rx.try_recv() {
                let rms = compute_rms(&chunk);
                recent_rms.push(rms);
            }

            if recent_rms.len() > 64 {
                recent_rms.drain(..recent_rms.len().saturating_sub(64));
            }

            let audio_active = recent_rms.iter().any(|&r| r > RMS_THRESHOLD);
            let trigger = meeting_detected && audio_active;

            if trigger && !sidebar_open {
                let _ = app_handle.emit("open-sidebar", ());
                sidebar_open = true;
            }

            if sidebar_open && !trigger {
                tokio::pin!(let cooldown = sleep(SIDEBAR_COOLDOWN););
                let mut activity_resumed = false;

                loop {
                    tokio::select! {
                        _ = &mut cooldown => break,
                        Some(chunk) = audio_rx.recv() => {
                            let rms = compute_rms(&chunk);
                            recent_rms.push(rms);
                            if recent_rms.len() > 64 {
                                recent_rms.drain(..recent_rms.len().saturating_sub(64));
                            }
                            let still_active = recent_rms.iter().any(|&r| r > RMS_THRESHOLD);
                            if is_meeting_running(&mut system) && still_active {
                                activity_resumed = true;
                                break;
                            }
                        }
                    }
                }

                if activity_resumed {
                    sidebar_open = true;
                } else {
                    let meeting_gone = !is_meeting_running(&mut system);
                    if meeting_gone {
                        let _ = app_handle.emit("close-sidebar", ());
                    }
                    sidebar_open = false;
                    recent_rms.clear();
                }
            }

            sleep(PROCESS_CHECK_INTERVAL).await;
        }
    });

    flag
}
