use std::{io, fs};
use std::path::PathBuf;
use chrono::{DateTime, Datelike, Timelike};
use derivative::Derivative;
use id3::{Tag, TagLike, Timestamp};

// Maybe I should split into headless and "extended"? Idk, I think it's gonna be fine once it's gonna be pulled from DB
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Song {
    pub path: PathBuf,
    pub derived_name: String,
    pub title: Option<String>,
    pub artists: Option<Vec<String>>,
    pub album: Option<String>,
    pub duration: Option<u32>,
    pub date: Option<Timestamp>,
    #[derivative(Debug = "ignore")]
    pub album_art: Option<Vec<u8>>,  // <-- Just image data. Let's hope egui don't choke on it
}

impl Song {
    pub fn from_path(path: PathBuf) -> Option<Song> {
        let filename = path.file_stem()?.to_str()?.to_string();

        match Tag::read_from_path(&path) {
            Ok(tag) => {
                let title = if let Some(title) = tag.title() {
                    Some(title.to_string())
                } else {
                    None
                };

                let artists = if let Some(title) = tag.artists() {
                    Some(title.iter().map(|x| x.to_string()).collect())
                } else {
                    None
                };

                let album = if let Some(title) = tag.album() {
                    Some(title.to_string())
                } else {
                    None
                };

                let duration = if let Some(duration) = tag.duration() {
                    Some(duration)
                } else {
                    None
                };

                let date = if let Some(date) = tag.date_recorded() {
                    Some(date)
                } else if let Some(date) = tag.date_released() {
                    Some(date)
                } else if let Some(date) = tag.original_date_released() {
                    Some(date)
                } else {
                    None
                };

                // Do some formatting
                let date = if let Some(date) = date {
                    if date.year < 10000 {
                        // It's fine as is
                        Some(date)
                    } else if date.year < 100000000 {
                        // yt-dlp formatting of YYYYMMDD
                        let year = date.year / 10000;
                        let month = (date.year % 10000) / 100;
                        let day = date.year % 100;
                        let timestamp = Timestamp {
                            year,
                            month: Some(month as u8),
                            day: Some(day as u8),
                            hour: None,
                            minute: None,
                            second: None,
                        };

                        Some(timestamp)
                    } else {
                        // Probably just a timestamp
                        if let Some(dt) = DateTime::from_timestamp(date.year as i64, 0) {
                            Some(Timestamp {
                                year: dt.year(),
                                month: Some(dt.month() as u8),
                                day: Some(dt.day() as u8),
                                hour: Some(dt.hour() as u8),
                                minute: Some(dt.minute() as u8),
                                second: Some(dt.second() as u8),
                            })
                        } else {
                            // Nope, something fucked up royally, just pass the thing as is
                            Some(date)
                        }
                    }
                } else {
                    None
                };

                let album_art = if let Some(picture) = tag.pictures().next() {
                    Some(picture.data.clone())
                } else {
                    None
                };

                Some(Song {
                    path,
                    derived_name: filename,
                    title,
                    artists,
                    album,
                    duration,
                    date,
                    album_art,
                })
            }
            Err(e) => {
                Some(Song {
                    path,
                    derived_name: filename,
                    title: None,
                    artists: None,
                    album: None,
                    duration: None,
                    date: None,
                    album_art: None,
                })
            }
        }
    }

    pub fn display(&self) -> String {
        match (&self.artists, &self.title) {
            (Some(artists), Some(title)) => {
                artists.join(", ") + " - " + &title
            }
            (None, Some(title)) => {title.to_string()}
            (Some(_), None) | (None, None) => {
                self.derived_name.clone()
            }
        }
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
