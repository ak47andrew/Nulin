use anyhow::anyhow;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use lofty::tag::items::Timestamp;
use std::path::PathBuf;
use std::{fs, io};

use crate::db::{DBHandler, FromDBRow, load_lofty_timestamp};

#[derive(Debug)]
pub struct Song {
    pub path: PathBuf,
    pub derived_name: String,
    pub title: Option<String>,
    pub artists: Option<Vec<String>>,
    pub album: Option<String>,
    pub duration: u32,
    pub date: Option<Timestamp>,
}

impl FromDBRow for Song {
    fn from_row(row: &rusqlite::Row) -> anyhow::Result<Self> {
        let path = PathBuf::from(row.get::<_, String>(1)?);
        let filename = path
            .file_stem()
            .ok_or(anyhow!("Failed to get file_stem"))?
            .to_str()
            .ok_or(anyhow!("Failed to covert to string"))?
            .to_string();

        Ok(Song {
            path: path,
            derived_name: filename,
            title: row.get(2)?,
            artists: row.get::<_, Option<String>>(3)?.map(|x| vec![x]),
            album: row.get(4)?,
            duration: row.get(5)?,
            date: load_lofty_timestamp(row.get(6)?),
        })
    }
}

impl Song {
    pub fn get_art(&self) -> Option<Vec<u8>> {
        let tagged_file = Probe::open(self.path.clone()).ok()?.read().ok()?;
        if let Some(tag) = tagged_file.primary_tag() {
            tag.pictures().iter().next().map(|x| x.data().to_vec())
        } else {
            None
        }
    }

    pub fn from_path(path: PathBuf) -> Option<Song> {
        let filename = path.file_stem()?.to_str()?.to_string();

        let tagged_file = Probe::open(path.clone()).ok()?.read().ok()?;
        let properties = tagged_file.properties();
        let duration = properties.duration().as_millis() as u32;

        if let Some(tag) = tagged_file.primary_tag() {
            let title = tag.title().map(|x| x.to_string());
            let album = tag.album().map(|x| x.to_string());
            let mut artists = tag
                .get_strings(lofty::tag::ItemKey::TrackArtists)
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            if artists.is_empty() {
                // Let's check jic ItemKey::TrackArtist is there
                if let Some(artist) = tag.get_string(lofty::tag::ItemKey::TrackArtist) {
                    artists = vec![artist.to_string()];
                }
            }
            let date = tag.date();
            println!("{:?}", date);
            // date.map(|x| x);
            Some(Song {
                path,
                derived_name: filename,
                title,
                artists: if !artists.is_empty() {
                    Some(artists)
                } else {
                    None
                },
                album,
                duration,
                date,
            })
        } else {
            Some(Song {
                path,
                derived_name: filename,
                title: None,
                album: None,
                artists: None,
                duration,
                date: None,
            })
        }
    }

    pub fn display(&self) -> String {
        match (&self.artists, &self.title) {
            (Some(artists), Some(title)) => artists.join(", ") + " - " + &title,
            (None, Some(title)) => title.to_string(),
            (Some(_), None) | (None, None) => self.derived_name.clone(),
        }
    }
}

const SUPPORTED_EXTENSIONS: [&str; 4] = ["mp3", "ogg", "wav", "flac"];
pub fn load_songs(path: &str, is_recursive: bool) -> io::Result<Vec<Song>> {
    // All DB stuff is fully for testing (probably... idk yet)
    let db = DBHandler::new().unwrap();

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

                // &&* is crazy xDD
                if !SUPPORTED_EXTENSIONS.contains(&&*extension.unwrap().to_lowercase()) {
                    continue;
                }

                let song = Song::from_path(path).unwrap();
                db.insert_song(&song).unwrap();
                files.push(song);
            }
        }
    }

    files.sort_by_cached_key(|x| x.display());

    Ok(files)
}
