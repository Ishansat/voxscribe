use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct VadConfig {
    pub sample_rate: u32,
    pub threshold: f32,
    pub min_speech_duration_ms: f64,
    pub max_speech_duration_ms: f64,
    pub hangover_ms: f64,
    pub pre_speech_pad_ms: f64,
    pub adaptive_noise_floor: bool,
    pub noise_floor_rise_rate: f32,
    pub threshold_multiplier: f32,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16_000,
            threshold: 0.02,
            min_speech_duration_ms: 150.0,
            max_speech_duration_ms: 3000.0,
            hangover_ms: 350.0,
            pre_speech_pad_ms: 120.0,
            adaptive_noise_floor: true,
            noise_floor_rise_rate: 0.005,
            threshold_multiplier: 2.5,
        }
    }
}

pub struct SpeechSegment {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

pub struct VoiceActivityDetector {
    config: VadConfig,
    pre_pad_buffer: VecDeque<f32>,
    speech_buffer: Vec<f32>,
    hangover_samples: usize,
    is_speech: bool,
    noise_floor: f32,
    hangover_max_samples: usize,
    pre_pad_max_samples: usize,
    min_speech_samples: usize,
    max_speech_samples: usize,
}

impl VoiceActivityDetector {
    pub fn new(config: VadConfig) -> Self {
        let sr = config.sample_rate as f64;
        Self {
            pre_pad_buffer: VecDeque::with_capacity(
                (config.pre_speech_pad_ms / 1000.0 * sr) as usize + 512,
            ),
            speech_buffer: Vec::new(),
            hangover_samples: 0,
            is_speech: false,
            noise_floor: 0.001,
            hangover_max_samples: (config.hangover_ms / 1000.0 * sr) as usize,
            pre_pad_max_samples: (config.pre_speech_pad_ms / 1000.0 * sr) as usize,
            min_speech_samples: (config.min_speech_duration_ms / 1000.0 * sr) as usize,
            max_speech_samples: (config.max_speech_duration_ms / 1000.0 * sr) as usize,
            config,
        }
    }

    pub fn push_frame(&mut self, frame: &[f32]) -> Option<SpeechSegment> {
        let rms = compute_rms(frame);

        let threshold = if self.config.adaptive_noise_floor {
            self.update_noise_floor(rms);
            (self.noise_floor * self.config.threshold_multiplier)
                .max(self.config.threshold * 0.25)
        } else {
            self.config.threshold
        };

        let frame_has_speech = rms > threshold;

        if !self.is_speech {
            self.pre_pad_buffer.extend(frame);
            while self.pre_pad_buffer.len() > self.pre_pad_max_samples {
                self.pre_pad_buffer.pop_front();
            }

            if frame_has_speech {
                self.is_speech = true;
                self.hangover_samples = 0;
                self.speech_buffer.clear();
                self.speech_buffer.extend(self.pre_pad_buffer.iter());
                self.speech_buffer.extend(frame);
            }

            None
        } else {
            self.speech_buffer.extend(frame);

            if frame_has_speech {
                self.hangover_samples = 0;
            } else {
                self.hangover_samples += frame.len();
            }

            let total_samples = self.speech_buffer.len();
            let hangover_expired = self.hangover_samples >= self.hangover_max_samples;
            let max_duration_reached = total_samples >= self.max_speech_samples;

            if hangover_expired || max_duration_reached {
                let result = if total_samples >= self.min_speech_samples {
                    Some(SpeechSegment {
                        samples: self.speech_buffer.clone(),
                        sample_rate: self.config.sample_rate,
                    })
                } else {
                    None
                };

                if max_duration_reached && frame_has_speech {
                    let overflow = total_samples.saturating_sub(self.max_speech_samples);
                    let split = self.speech_buffer.len().saturating_sub(overflow);
                    let remainder: Vec<f32> = self.speech_buffer.drain(split..).collect();
                    self.speech_buffer = remainder;
                    self.is_speech = true;
                    self.hangover_samples = 0;
                } else {
                    self.is_speech = false;
                    self.speech_buffer.clear();
                }

                result
            } else {
                None
            }
        }
    }

    pub fn flush(&mut self) -> Option<SpeechSegment> {
        if self.is_speech && self.speech_buffer.len() >= self.min_speech_samples {
            self.is_speech = false;
            Some(SpeechSegment {
                samples: self.speech_buffer.clone(),
                sample_rate: self.config.sample_rate,
            })
        } else {
            None
        }
    }

    fn update_noise_floor(&mut self, rms: f32) {
        if rms < self.noise_floor {
            self.noise_floor = rms;
        } else {
            self.noise_floor +=
                self.config.noise_floor_rise_rate * (rms - self.noise_floor);
        }
        self.noise_floor = self.noise_floor.max(1e-8);
    }
}

fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}
