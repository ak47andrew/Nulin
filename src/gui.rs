use egui::*;
use egui_alignments::*;
use crate::widgets::*;

#[derive(PartialEq)]
pub enum AppScreen {
    Current,
    Artists
}

pub struct CounterApp {
    screen: AppScreen,
    value: i32,
    volume: f32,
}

impl CounterApp {
    fn draw(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        Panel::bottom("player").min_size(70.0).show(ui, |ui| {
            ui.scope(|ui| {
                ui.style_mut().spacing.slider_width = ui.available_width();
                ui.add(Slider::new(&mut self.value, 0..=1000).show_value(false).trailing_fill(true));
            });

            let row_rect = ui.available_rect_before_wrap();

            center_horizontal(ui, |ui| {
                if circular_button(ui, 50.0, RichText::new("<").size(35.0)).clicked() {println!("Prev"); self.value -= 1;}
                if circular_button(ui, 50.0, RichText::new("II").size(35.0)).clicked() {println!("Play"); self.value = 50;}
                if circular_button(ui, 50.0, RichText::new(">").size(35.0)).clicked() {println!("Next"); self.value += 1;}
            });

            // 3. Define a fixed width for the volume slider UI
            let volume_ui_width = 150.0;

            // 4. Carve out a rectangle pinned to the absolute right side of the row
            let right_slider_rect = Rect::from_min_max(
                pos2(row_rect.max.x - volume_ui_width, row_rect.min.y),
                pos2(row_rect.max.x, row_rect.max.y),
            );

            ui.scope(|ui| {
                ui.style_mut().spacing.slider_width = 140.0;
                ui.put(right_slider_rect, Slider::new(&mut self.volume, 0.0..=100.0).show_value(false).trailing_fill(true));
            });
        });

        Panel::left("sidebar").resizable(true).default_size(150.0).size_range(150.0..=250.0).show(ui, |ui| {
            ui.take_available_width();
            ui.horizontal(|ui| {
                ui.add_space(40.0);  // Fragile, but nothing else works for whatever the fuck reason
                ui.heading("Labels");
            });
            sidebar_entry(ui, &mut self.screen, AppScreen::Current, "Player");
            sidebar_entry(ui, &mut self.screen, AppScreen::Artists, "Artists");
        });

        CentralPanel::default().show(ui, |ui| {
            match self.screen {
                AppScreen::Current => {self.draw_current(ui, _frame)}
                AppScreen::Artists => {self.draw_artists(ui, _frame)}
            }
        });
    }

    fn draw_current(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.heading("Currently playing");
        ui.label("Value");
        ui.label(self.value.to_string());
        ui.label("Example");
        ui.label("Example values");
    }

    fn draw_artists(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.heading("Artists");
        ui.label("Value");
        ui.label(self.volume.to_string());
        ui.label("Example");
        ui.label("Example values");
    }
}

impl Default for CounterApp {
    fn default() -> Self {
        Self { value: 0, volume: 0.0, screen: AppScreen::Current }
    }
}

impl eframe::App for CounterApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            ui.send_viewport_cmd(ViewportCommand::Close);
        }
        self.draw(ui, _frame);
    }
}
