use std::env;
use std::path::{Path, PathBuf};

pub struct ModelLoader;

impl ModelLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn find_model(&self, filename: &str) -> Option<PathBuf> {
        let home = env::var("HOME").ok()?;

        let candidates = [
            Path::new("models").join(filename),
            Path::new("../models").join(filename),
            Path::new(&home).join(".local/share/voxscribe/models").join(filename),
            Path::new(&home).join("Library/Application Support/voxscribe/models").join(filename),
        ];

        for path in &candidates {
            if path.exists() {
                return Some(path.to_path_buf());
            }
        }

        None
    }

    pub fn load_model(&self) -> Result<(), String> {
        Ok(())
    }
}
