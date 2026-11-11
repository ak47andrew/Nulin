use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{create_dir_all, read_to_string};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SaveState {
    pub shuffle: bool,
    pub loop_one: bool,

    // Read and updated only after loading and before saving
    pub volume: f32,
    pub song_id: Option<usize>,
    pub progress_ms: u64,
    pub search_input: String,
}

impl Default for SaveState {
    fn default() -> Self {
        SaveState {
            shuffle: false,
            loop_one: false,
            volume: 0.5,
            song_id: None,
            progress_ms: 0,
            search_input: String::new(),
        }
    }
}

impl SaveState {
    pub fn load_or_default() -> Self {
        let path = SaveState::get_path();
        let content = match read_to_string(path) {
            Ok(content) => content,
            Err(_) => {
                println!("File not found, using default");
                return SaveState::default();
            }
        };
        toml::from_str::<SaveState>(&content).unwrap_or_else(|_| {
            println!("Failed to deserialize, using default");
            let s = SaveState::default();
            s.save();
            s
        })
    }

    pub fn save(&self) {
        let path = SaveState::get_path();
        let toml_string = match toml::to_string_pretty(self) {
            Ok(toml_string) => toml_string,
            Err(_) => {
                println!("Failed to serialize");
                return;
            }
        };
        fs::write(path, toml_string).unwrap();
    }

    fn get_path() -> PathBuf {
        get_base_path().join("savestate.toml")
    }
}

pub fn get_base_path() -> PathBuf {
    let dirs = ProjectDirs::from("dev", "reborn", "nulin").unwrap();
    let config_dir = dirs.config_dir();
    create_dir_all(config_dir).unwrap();
    config_dir.to_path_buf()
}
