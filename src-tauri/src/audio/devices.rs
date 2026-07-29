use cpal::traits::{DeviceTrait, HostTrait};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum AudioDeviceType {
    Microphone,
    Loopback,
}

#[derive(Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub device_type: AudioDeviceType,
    pub(crate) device: cpal::Device,
}

impl fmt::Debug for AudioDeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioDeviceInfo")
            .field("name", &self.name)
            .field("device_type", &self.device_type)
            .finish()
    }
}

impl AudioDeviceInfo {
    pub fn from_device(device: cpal::Device) -> Self {
        let name = device.name().unwrap_or_else(|_| "Unknown".into());
        let device_type = if is_loopback_name(&name) {
            AudioDeviceType::Loopback
        } else {
            AudioDeviceType::Microphone
        };
        Self { name, device_type, device }
    }

    pub fn default_config(&self) -> Result<cpal::SupportedStreamConfig, anyhow::Error> {
        Ok(self.device.default_input_config()?)
    }
}

const LOOPBACK_PATTERNS: &[&str] = &[
    "blackhole", "loopback", "audio hijack", "soundflower",
    "virtual audio", "vb-cable", "hifi cable", "totalmix",
    "aggregate", "multi-output",
];

fn is_loopback_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    LOOPBACK_PATTERNS.iter().any(|kw| lower.contains(kw))
}

pub fn enum_input_devices() -> Vec<AudioDeviceInfo> {
    let host = cpal::default_host();
    match host.input_devices() {
        Ok(devices) => devices
            .filter_map(|d| {
                d.name().ok()?;
                Some(AudioDeviceInfo::from_device(d))
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub fn enum_output_devices() -> Vec<AudioDeviceInfo> {
    let host = cpal::default_host();
    match host.output_devices() {
        Ok(devices) => devices
            .filter_map(|d| {
                d.name().ok()?;
                Some(AudioDeviceInfo::from_device(d))
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub fn default_microphone() -> Option<AudioDeviceInfo> {
    let host = cpal::default_host();
    host.default_input_device()
        .map(|d| {
            let name = d.name().unwrap_or_default();
            let device_type = if is_loopback_name(&name) {
                AudioDeviceType::Loopback
            } else {
                AudioDeviceType::Microphone
            };
            AudioDeviceInfo { name, device_type, device: d }
        })
        .filter(|d| d.device_type == AudioDeviceType::Microphone)
}

pub fn default_loopback() -> Option<AudioDeviceInfo> {
    enum_input_devices().into_iter().find(|d| d.device_type == AudioDeviceType::Loopback)
}
