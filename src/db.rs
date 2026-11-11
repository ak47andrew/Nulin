use anyhow::{Result, bail};
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
             song.date.map(|x| x.to_string()) // This is basically working bc .to_string is ISO-8601. It's a hack, but well fuck
         ])?;

        Ok(())
    }
}
