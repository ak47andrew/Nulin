pub mod gui;
pub mod widgets;
pub mod song;
pub mod playback;
pub mod media_controls;

use std::time::Instant;
use crate::gui::App;
use crate::song::load_songs;

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
        eframe::run_native(
            "Nulin",
            options,
            Box::new(|_cc| Ok(Box::<App>::default())),
        )
    }
}
