use egui::*;
use egui_alignments::*;
use crate::song::{load_songs, Song};
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
    cover_image: Option<TextureHandle>,
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
                self.change_track(ui, -1);
            }
            if circular_button(ui, 50.0, RichText::new("II").size(35.0)).clicked() {
                self.set_track(ui, None);
            }
            if circular_button(ui, 50.0, RichText::new(">").size(35.0)).clicked() {
                // Here modulo just works
                self.change_track(ui, 1);
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
        if let Some(texture_handle) = &self.cover_image && let Some(idx) = self.selected_idx {
            let song = &self.songs[idx];
            let available_size = ui.available_size() / 1.5; // Returns an egui::Vec2 (width, height)

            // 2. Get the texture size as floats and its aspect ratio
            let texture_size = texture_handle.size_vec2();
            let aspect_ratio = texture_handle.aspect_ratio();

            // 3. Prevent division by zero if the texture hasn't loaded properly
            let (w_0, h_0) = if texture_size.y > 0.0 && texture_size.x > 0.0 {
                // Check if scaling by height makes the width too wide
                if aspect_ratio * available_size.y <= available_size.x {
                    // Height is the limiting factor
                    (aspect_ratio * available_size.y, available_size.y)
                } else {
                    // Width is the limiting factor
                    (available_size.x, available_size.x / aspect_ratio)
                }
            } else {
                (0.0, 0.0)
            };

            center_vertical(ui, |ui| {
                ui.add(Image::new(texture_handle).max_size(vec2(w_0, h_0)));
                ui.add(Label::new(RichText::new(song.display()).size(20.0)))
            });
        }
    }

    fn draw_cental_songs(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let mut new_ind = None;
        ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                for (ind, song) in self.songs.iter().enumerate() {
                    let resp = sized_entry(ui, 40.0, Align2::LEFT_TOP, self.selected_idx == Some(ind), &song.derived_name);
                    if resp.clicked() {
                        new_ind = Some(ind);  // Just to make borrow checker shut the fuck up
                    }
                    if resp.double_clicked() {
                        println!("{:?}", self.songs[self.selected_idx.unwrap()]);
                        self.screen = AppScreen::Current;
                    }
                }
            });
        if let Some(ind) = new_ind {
            self.set_track(ui, Some(ind));
        }
    }

    fn set_track(&mut self, ui: &mut Ui, idx: Option<usize>) {
        self.selected_idx = idx;
        if idx.is_none() {
            self.cover_image = None;
            return;
        }
        let idx = self.selected_idx.unwrap();
        let song = &self.songs[idx];
        let img = if let Some(art) = &song.album_art {
            image::load_from_memory(art).unwrap()
        } else {
            image::load_from_memory(include_bytes!("../assets/missing-draft.png")).unwrap()
        };

        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];

        let color_image = ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
        let texture_handle = ui.load_texture(
            "cover_art",
            color_image,
            TextureOptions::default()
        );
        self.cover_image = Some(texture_handle);
    }

    fn change_track(&mut self, ui: &mut Ui, offset: i32) {
        // Calculate next index
        let mut idx = match self.selected_idx {
            Some(idx) => idx,
            None => return,  // Moving None doesn't do shit, so we don't care in the slightest
        };
        if offset < 0 && idx as i32 + offset < 0 {
            idx = (self.songs.len() as i32 + idx as i32 + offset) as usize;
        } else {
            idx = (idx as i32 + offset) as usize % self.songs.len();
        }

        self.set_track(ui, Some(idx));
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            value: 0.0,
            volume: 0.0,
            screen: AppScreen::Current,
            songs: load_songs(if option_env!("FULL_SONG_LIST") == Some("1") {
                "/home/tumpa/projects/sandbox/music_downloads/"
            } else {
                "songs"
            }, false).unwrap(),
            selected_idx: None,
            cover_image: None
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
