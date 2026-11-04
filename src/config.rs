use std::fs;
use std::fs::{create_dir_all, read_to_string};
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub shuffle: bool,
    pub loop_one: bool,

    // Read and updated only after loading and before saving
    pub volume: f32,
    pub dir: PathBuf,
    pub song_id: Option<usize>,
    pub progress_ms: u64
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dir: PathBuf::new(),
            shuffle: false,
            loop_one: false,
            volume: 0.5,
            song_id: None,
            progress_ms: 0
        }
    }
}

impl Config {
    pub fn load_or_default() -> Self {
        let path = Config::get_path();
        let content = match read_to_string(path) {
            Ok(content) => content,
            Err(_) => {println!("File not found, using default"); return Config::default()}
        };
        toml::from_str::<Config>(&content).unwrap_or_else(|_| { println!("Failed to deserialize, using default"); Config::default() })
    }

    pub fn save(&self) {
        let path = Config::get_path();
        let toml_string = match toml::to_string_pretty(self) {
            Ok(toml_string) => toml_string,
            Err(_) => {println!("Failed to serialize"); return }
        };
        fs::write(path, toml_string).unwrap();
    }

    fn get_path() -> PathBuf {
        let dirs = ProjectDirs::from("dev", "reborn", "nulin").unwrap();
        let config_dir = dirs.config_dir();
        create_dir_all(config_dir).unwrap();
        config_dir.to_path_buf().with_file_name("nulin.toml")
    }
}