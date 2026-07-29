pub struct AudioDeviceInfo {
    pub name: String,
    pub is_loopback: bool,
}

pub fn discover_devices() -> Vec<AudioDeviceInfo> {
    Vec::new()
}
