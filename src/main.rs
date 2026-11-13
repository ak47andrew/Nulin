pub mod config;
pub mod db;
pub mod gui;
pub mod media_controls;
pub mod playback;
pub mod song;
pub mod widgets;

use crate::gui::App;
use crate::song::{Song, load_songs};
use std::time::Instant;

fn main() -> eframe::Result {
    if option_env!("PERFORMANCE_TEST") == Some("1") {
        let start = Instant::now();
        let songs = load_songs("/home/tumpa/projects/sandbox/music_downloads/", true).unwrap();
        let elapsed = start.elapsed().as_secs_f32();
        for song in &songs {
            println!("{:?}", song)
        }
        println!("Loaded {} songs in {}", songs.len(), elapsed);
        Ok(())
    } else {
        let options = eframe::NativeOptions::default();
        eframe::run_native("Nulin", options, Box::new(|_cc| Ok(Box::<App>::default())))
    }
}
