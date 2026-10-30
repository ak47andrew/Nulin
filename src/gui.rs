use egui::{Button, CentralPanel, Key, Label, Modifiers, Panel, RichText};
use egui_alignments::center_horizontal;
use crate::widgets::circular_button;

pub enum AppScreen {
    Regular,
    Maxed
}

pub struct CounterApp {
    screen: AppScreen,
    value: i32,
}

impl CounterApp {
    fn draw_regular(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        Panel::bottom("player").min_size(70.0).show(ui, |ui| {
            center_horizontal(ui, |ui| {
                if circular_button(ui, 50.0, RichText::new("<").size(35.0)).clicked() {println!("Prev")}
                if circular_button(ui, 50.0, RichText::new("II").size(35.0)).clicked() {println!("Play")}
                if circular_button(ui, 50.0, RichText::new(">").size(35.0)).clicked() {println!("Next")}
            });
        });

        Panel::left("sidebar").resizable(true).default_size(150.0).size_range(150.0..=250.0).show(ui, |ui| {
            ui.take_available_width();
            ui.label("Value");
            ui.label(self.value.to_string());
            ui.label("Example");
            ui.label("Example values");
        });

        CentralPanel::default().show(ui, |ui| {
            ui.heading("Central Panel");
            ui.label("1");
            ui.label("342");
        });
    }

    fn draw_maxed(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.add(Label::new(egui::RichText::new(self.value.to_string()).size(300.0)));
            });

            if ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::T)) {
                self.screen = AppScreen::Regular;
            }
        });
    }
}

impl Default for CounterApp {
    fn default() -> Self {
        Self { value: 0, screen: AppScreen::Regular }
    }
}

impl eframe::App for CounterApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        match self.screen {
            AppScreen::Regular => {self.draw_regular(ui, _frame)}
            AppScreen::Maxed => {self.draw_maxed(ui, _frame)}
        }
    }
}
