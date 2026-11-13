use anyhow::{Result, bail};
use chrono::{DateTime, Datelike, NaiveDate, SecondsFormat, Timelike};
use lofty::tag::items::Timestamp;
use rusqlite::{Connection, Error as SqliteError, Params, Row, params};

use crate::{config::get_base_path, song::Song};

pub struct DBHandler {
    conn: Connection,
}

pub trait FromDBRow: Sized {
    fn from_row(row: &Row) -> Result<Self>;
}

impl DBHandler {
    pub fn new() -> Result<Self> {
        let conn = Connection::open(get_base_path().join("nulin.db"))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        let obj = Self { conn };

        if obj
            .post(
                r#"CREATE TABLE IF NOT EXISTS "songs" (
        	    "song_id"	INTEGER,
            	"path"	TEXT NOT NULL UNIQUE,
            	"title"	TEXT,
            	"artists"	TEXT,
            	"album"	TEXT,
            	"duration"	INTEGER,
            	"date"	INTEGER,
            	PRIMARY KEY("song_id" AUTOINCREMENT)
            );"#,
                [],
            )
            .is_err()
        {
            bail!("Failed to create \"Songs\" table");
        }

        Ok(obj)
    }

    pub fn post<P: Params>(&self, command: &str, params: P) -> Result<()> {
        self.conn.execute(command, params)?;
        Ok(())
    }

    pub fn get<P, T, F>(&self, command: &str, params: P, f: F) -> Result<Vec<T>>
    where
        P: Params,
        F: Fn(&Row) -> Result<T>,
    {
        let mut stmt = self.conn.prepare(command)?;
        let rows = stmt.query_map(params, |row| {
            f(row).map_err(|err| SqliteError::ToSqlConversionFailure(err.into()))
        })?;

        rows.map(|r| r.map_err(anyhow::Error::from)).collect()
    }

    pub fn get_to_obj<P: Params, T: FromDBRow>(&self, command: &str, params: P) -> Result<Vec<T>> {
        self.get(command, params, T::from_row)
    }

    pub fn insert_song(&self, song: &Song) -> Result<()> {
        self.post(r#"INSERT INTO songs (path, title, artists, album, duration, date) VALUES (?, ?, ?, ?, ?, ?)"#, params![
             song.path.to_str().unwrap(),
             song.title,
             song.artists.clone().map(|x| x.join(", ")),
             song.album,
             song.duration,
             // song.date.map(|x| x.to_string()) // This is basically working bc .to_string is ISO-8601. It's a hack, but well fuck
             // Upd: well, guess what... it's not! This piece of ass, even tho documented to save to ISO-8601, doesn't do that.
             // Yandex Music downloads only returned with a year so it was saved as "2024" and breaks fucking everything on load.
             // As GTA 5 devs said: "Stupid fuck"
             // I'm just gonna save/load timestamps and get `chrono` back into place
            song.date.map(|x| save_lofty_timestamp(&x))
         ])?;

        Ok(())
    }

    pub fn get_song(&self, id: usize) -> Option<Song> {
        let s = self
            .get_to_obj("SELECT * FROM songs WHERE song_id = ?", params![id as u32])
            .ok()?;
        s.into_iter().next()
    }

    pub fn get_songs(&self) -> Vec<Song> {
        // TODO: return SongHeadless and make .extend() because fetching 100MB of data from 3k songs is crazy.
        // And this one only used for a search and song listings
        self.get_to_obj("SELECT * FROM songs", []).unwrap_or(vec![])
    }

    pub fn get_songs_len(&self) -> usize {
        self.get("SELECT COUNT(*) FROM songs", [], |row| {
            Ok(row.get::<_, u32>(0)?)
        })
        .unwrap_or(vec![0])[0] as usize
    }
}

pub fn save_lofty_timestamp(timestamp: &Timestamp) -> i64 {
    let year = timestamp.year as i32;
    let month = timestamp.month.unwrap_or(1) as u32;
    let day = timestamp.day.unwrap_or(1) as u32;
    let hour = timestamp.hour.unwrap_or(0) as u32;
    let minute = timestamp.minute.unwrap_or(0) as u32;
    let second = timestamp.second.unwrap_or(0) as u32;

    let actually_usefull_timestamp_thank_you_chrono = NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(hour, minute, second)
        .unwrap()
        .and_utc();

    actually_usefull_timestamp_thank_you_chrono.timestamp()
}

pub fn load_lofty_timestamp(secs: Option<i64>) -> Option<Timestamp> {
    match secs {
        None => None,
        Some(seconds) => {
            let dt = DateTime::from_timestamp_secs(seconds)?;
            Some(Timestamp {
                year: dt.year() as u16,
                month: Some(dt.month() as u8),
                day: Some(dt.day() as u8),
                hour: Some(dt.hour() as u8),
                minute: Some(dt.minute() as u8),
                second: Some(dt.second() as u8),
            })
        }
    }
}
