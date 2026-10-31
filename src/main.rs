pub mod gui;
pub mod widgets;

use crate::gui::CounterApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Nulin",
        options,
        Box::new(|_cc| Ok(Box::<CounterApp>::default())),
    )
}
