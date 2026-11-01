// TODO: rename this file. I'm super tired (for some reason it's, like, 6:30PM) and don't remember how to speak
use std::{io, fs};
use std::path::PathBuf;

// Maybe I should split into headless and "extended"? Idk, I think it's gonna be fine once it's gonna be pulled from DB
pub struct Song {
    pub path: PathBuf,
    pub derived_name: String,
}

impl Song {
    pub fn from_path(path: PathBuf) -> Option<Song> {
        let filename = path.file_stem()?.to_str()?.to_string();

        Some(Song {
            path,
            derived_name: filename,
        })
    }
}

const SUPPORTED_EXTENSIONS: [&str; 4] = ["mp3", "ogg", "wav", "flac"];
pub fn load_songs(path: &str, is_recursive: bool) -> io::Result<Vec<Song>> {
    let mut files = vec![];
    let mut dirs = vec![PathBuf::from(path)];

    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && is_recursive {
                dirs.push(path);
                continue;
            }
            if path.is_file() {
                let extension = path.extension();
                if extension.is_none() {
                    continue;
                }

                let extension = extension.unwrap().to_str();
                if extension.is_none() {
                    continue;
                }

                if !SUPPORTED_EXTENSIONS.contains(&extension.unwrap()) {
                    continue;
                }

                files.push(Song::from_path(path).unwrap());
            }
        }
    }

    Ok(files)
}
