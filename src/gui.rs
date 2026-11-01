use egui::*;
use egui_alignments::*;
use crate::stuff::{load_songs, Song};
use crate::widgets::*;

#[derive(PartialEq)]
pub enum AppScreen {
    Current,
    Songs
}

pub struct App {
    screen: AppScreen,
    value: f32,
    volume: f32,
    songs: Vec<Song>,

    // AppScreen variables (it just works better if App is gonna hold all this)
    selected_idx: Option<usize>,
}

impl App {
    fn draw(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        Panel::bottom("player").min_size(70.0).show(ui, |ui| {
            self.draw_player(ui);
        });

        Panel::left("sidebar")
            .resizable(true)
            .default_size(150.0)
            .size_range(150.0..=250.0)
            .show(ui, |ui| {
                self.draw_sidebar(ui);
            });

        CentralPanel::default().show(ui, |ui| {
            match self.screen {
                AppScreen::Current => {self.draw_central_current(ui, _frame)}
                AppScreen::Songs => {self.draw_cental_songs(ui, _frame)}
            }
        });

        ui.request_repaint();
    }

    fn draw_player(&mut self, ui: &mut Ui) {
        ui.scope(|ui| {
            ui.style_mut().spacing.slider_width = ui.available_width();
            ui.add(Slider::new(&mut self.value, 0.0..=1.0).show_value(false).trailing_fill(true));
        });

        let row_rect = ui.available_rect_before_wrap();

        center_horizontal(ui, |ui| {
            if circular_button(ui, 50.0, RichText::new("<").size(35.0)).clicked() {
                if let Some(idx) = self.selected_idx {
                    // Edge case bc (idx - 1) might underflow and panic
                    if idx == 0 {
                        self.selected_idx = Some(self.songs.len() - 1);
                    } else {
                        self.selected_idx = Some(idx - 1);
                    }
                }
            }
            if circular_button(ui, 50.0, RichText::new("II").size(35.0)).clicked() {
                self.selected_idx = None;
            }
            if circular_button(ui, 50.0, RichText::new(">").size(35.0)).clicked() {
                // Here modulo just works
                if let Some(idx) = self.selected_idx {
                    self.selected_idx = Some((idx + 1) % self.songs.len());
                }
            }
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
    }

    fn draw_sidebar(&mut self, ui: &mut Ui) {
        ui.take_available_width();
        ui.horizontal(|ui| {
            ui.add_sized([ui.available_width(), 20.0], Label::new(RichText::new("Labels").heading()));
        });
        sidebar_entry(ui, &mut self.screen, AppScreen::Current, "Player");
        sidebar_entry(ui, &mut self.screen, AppScreen::Songs, "Songs");
    }

    // Central Panel modes
    fn draw_central_current(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.heading("Currently playing");
        ui.label("Value");
        ui.label(self.value.to_string());
        ui.label("Example");
        ui.label("Example values");
    }

    fn draw_cental_songs(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                for (ind, song) in self.songs.iter().enumerate() {
                    let resp = sized_entry(ui, 40.0, Align2::LEFT_TOP, self.selected_idx == Some(ind), &song.derived_name);
                    if resp.clicked() {
                        self.selected_idx = Some(ind);
                    }
                }
            });
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            value: 0.0,
            volume: 0.0,
            screen: AppScreen::Current,
            songs: load_songs("songs", false).unwrap(),
            selected_idx: None,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            ui.send_viewport_cmd(ViewportCommand::Close);
        }
        self.draw(ui, _frame);
    }
}
