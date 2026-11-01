pub mod gui;
pub mod widgets;
pub mod stuff;

use crate::gui::App;
use crate::stuff::load_songs;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Nulin",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}
