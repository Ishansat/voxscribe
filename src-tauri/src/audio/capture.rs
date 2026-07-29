use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, StreamTrait};
use tokio::sync::mpsc as tokio_mpsc;

use super::devices::AudioDeviceInfo;

const TARGET_SAMPLE_RATE: u32 = 16_000;
const DEFAULT_CHUNK_MS: f64 = 30.0;

#[derive(Clone, Debug)]
pub struct AudioCaptureConfig {
    pub target_sample_rate: u32,
    pub chunk_duration_ms: f64,
    pub mic_gain: f32,
    pub loopback_gain: f32,
}

impl Default for AudioCaptureConfig {
    fn default() -> Self {
        Self {
            target_sample_rate: TARGET_SAMPLE_RATE,
            chunk_duration_ms: DEFAULT_CHUNK_MS,
            mic_gain: 0.5,
            loopback_gain: 0.5,
        }
    }
}

pub struct AudioCapture {
    mic_device: AudioDeviceInfo,
    loopback_device: Option<AudioDeviceInfo>,
    config: AudioCaptureConfig,
}

impl AudioCapture {
    pub fn new(mic_device: AudioDeviceInfo, loopback_device: Option<AudioDeviceInfo>) -> Self {
        Self {
            mic_device,
            loopback_device,
            config: AudioCaptureConfig::default(),
        }
    }

    pub fn with_config(
        mic_device: AudioDeviceInfo,
        loopback_device: Option<AudioDeviceInfo>,
        config: AudioCaptureConfig,
    ) -> Self {
        Self {
            mic_device,
            loopback_device,
            config,
        }
    }

    pub fn start(self) -> Result<AudioStream, anyhow::Error> {
        let chunk_samples =
            (self.config.target_sample_rate as f64 * self.config.chunk_duration_ms / 1000.0) as usize;

        let (mic_tx, mic_rx) = mpsc::channel::<Vec<f32>>();
        let (loopback_tx, loopback_rx) = mpsc::channel::<Vec<f32>>();
        let (output_tx, output_rx) = tokio_mpsc::channel::<Vec<f32>>(64);

        let running = Arc::new(AtomicBool::new(true));

        let mic_stream = build_cpal_stream(&self.mic_device.device, mic_tx)?;
        mic_stream.play()?;

        let loopback_stream = match &self.loopback_device {
            Some(device) => {
                let stream = build_cpal_stream(&device.device, loopback_tx)?;
                stream.play()?;
                Some(stream)
            }
            None => None,
        };

        let mic_config = self.mic_device.default_config().ok();
        let loopback_config = self
            .loopback_device
            .as_ref()
            .and_then(|d| d.default_config().ok());

        let target_rate = self.config.target_sample_rate;
        let mic_gain = self.config.mic_gain;
        let loopback_gain = self.config.loopback_gain;
        let running_clone = running.clone();

        let mixer_handle = thread::Builder::new()
            .name("voxscribe-audio-mixer".into())
            .spawn(move || {
                let mut mic_resampler = mic_config
                    .as_ref()
                    .map(|c| Resampler::new(c.sample_rate().0, target_rate));
                let mut loopback_resampler = loopback_config
                    .as_ref()
                    .map(|c| Resampler::new(c.sample_rate().0, target_rate));

                let mut mic_buf: VecDeque<f32> = VecDeque::new();
                let mut loopback_buf: VecDeque<f32> = VecDeque::new();
                let mut mixed_buffer: Vec<f32> = Vec::with_capacity(chunk_samples);

                while running_clone.load(Ordering::Relaxed) {
                    if let Ok(chunk) = mic_rx.try_recv() {
                        if let Some(ref mut resampler) = mic_resampler {
                            let resampled = resampler.process(&chunk);
                            mic_buf.extend(resampled);
                        } else {
                            mic_buf.extend(chunk);
                        }
                    }

                    if let Some(ref rx) = loopback_rx {
                        if let Ok(chunk) = rx.try_recv() {
                            if let Some(ref mut resampler) = loopback_resampler {
                                let resampled = resampler.process(&chunk);
                                loopback_buf.extend(resampled);
                            } else {
                                loopback_buf.extend(chunk);
                            }
                        }
                    }

                    while mixed_buffer.len() < chunk_samples
                        && (!mic_buf.is_empty() || !loopback_buf.is_empty())
                    {
                        let mic_sample = mic_buf.pop_front().unwrap_or(0.0);
                        let loopback_sample = loopback_buf.pop_front().unwrap_or(0.0);
                        let mixed = (mic_sample * mic_gain + loopback_sample * loopback_gain)
                            .clamp(-1.0, 1.0);
                        mixed_buffer.push(mixed);
                    }

                    if mixed_buffer.len() >= chunk_samples {
                        let chunk = mixed_buffer.drain(..chunk_samples).collect();
                        let _ = output_tx.blocking_send(chunk);
                    } else {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            })?;

        Ok(AudioStream {
            _mic_stream: mic_stream,
            _loopback_stream: loopback_stream,
            _mixer_handle: Some(mixer_handle),
            _running: running,
            audio_rx: output_rx,
            target_sample_rate: target_rate,
        })
    }
}

fn build_cpal_stream(
    device: &cpal::Device,
    tx: mpsc::Sender<Vec<f32>>,
) -> Result<cpal::Stream, anyhow::Error> {
    let config = device.default_input_config()?;
    let channels = config.channels() as usize;
    let err_fn = |err: cpal::StreamError| {
        eprintln!("cpal stream error: {}", err);
    };

    match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let stream = device.build_input_stream::<f32, _, _>(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if channels > 1 {
                        let mono: Vec<f32> = data
                            .chunks(channels)
                            .map(|ch| ch.iter().sum::<f32>() / channels as f32)
                            .collect();
                        let _ = tx.send(mono);
                    } else {
                        let _ = tx.send(data.to_vec());
                    }
                },
                err_fn,
                None,
            )?;
            Ok(stream)
        }
        cpal::SampleFormat::I16 => {
            let stream = device.build_input_stream::<i16, _, _>(
                &config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = if channels > 1 {
                        data.chunks(channels)
                            .map(|ch| ch.iter().map(|&s| s as f32 / 32768.0).sum::<f32>() / channels as f32)
                            .collect()
                    } else {
                        data.iter().map(|&s| s as f32 / 32768.0).collect()
                    };
                    let _ = tx.send(mono);
                },
                err_fn,
                None,
            )?;
            Ok(stream)
        }
        cpal::SampleFormat::U16 => {
            let stream = device.build_input_stream::<u16, _, _>(
                &config.into(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = if channels > 1 {
                        data.chunks(channels)
                            .map(|ch| {
                                ch.iter()
                                    .map(|&s| (s as f32 - 32768.0) / 32768.0)
                                    .sum::<f32>()
                                    / channels as f32
                            })
                            .collect()
                    } else {
                        data.iter()
                            .map(|&s| (s as f32 - 32768.0) / 32768.0)
                            .collect()
                    };
                    let _ = tx.send(mono);
                },
                err_fn,
                None,
            )?;
            Ok(stream)
        }
        other => anyhow::bail!("unsupported cpal sample format: {:?}", other),
    }
}

struct Resampler {
    ratio: f64,
    phase: f64,
    buffer: VecDeque<f32>,
}

impl Resampler {
    fn new(input_rate: u32, output_rate: u32) -> Self {
        Self {
            ratio: output_rate as f64 / input_rate as f64,
            phase: 0.0,
            buffer: VecDeque::new(),
        }
    }

    fn process(&mut self, input: &[f32]) -> Vec<f32> {
        self.buffer.extend(input);
        let mut output = Vec::new();

        while self.buffer.len() >= 2 {
            let pos = self.phase;
            let idx = pos.floor() as usize;
            let frac = pos.fract() as f32;

            if idx + 1 >= self.buffer.len() {
                break;
            }

            let sample = self.buffer[idx] * (1.0 - frac) + self.buffer[idx + 1] * frac;
            output.push(sample);
            self.phase += self.ratio;
        }

        let consumed = self.phase.floor() as usize;
        if consumed > 0 && !self.buffer.is_empty() {
            let drain_len = consumed.min(self.buffer.len());
            self.buffer.drain(..drain_len);
            self.phase -= consumed as f64;
        }

        output
    }
}

pub struct AudioStream {
    _mic_stream: cpal::Stream,
    _loopback_stream: Option<cpal::Stream>,
    _mixer_handle: Option<thread::JoinHandle<()>>,
    _running: Arc<AtomicBool>,
    audio_rx: tokio_mpsc::Receiver<Vec<f32>>,
    target_sample_rate: u32,
}

impl AudioStream {
    pub fn receiver(&mut self) -> &mut tokio_mpsc::Receiver<Vec<f32>> {
        &mut self.audio_rx
    }

    pub fn sample_rate(&self) -> u32 {
        self.target_sample_rate
    }
}

impl Drop for AudioStream {
    fn drop(&mut self) {
        self._running.store(false, Ordering::Relaxed);
    }
}
