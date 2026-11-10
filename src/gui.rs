use crate::config::Config;
use crate::media_controls::{MediaController, start_souvlaki_thread};
use crate::playback::{Command, Event, PlayerHandle, spawn_player};
use crate::song::{Song, load_songs};
use crate::widgets::*;
use egui::*;
use egui_alignments::*;
use image::DynamicImage;
use rand::random_range;
use souvlaki::{MediaControlEvent, MediaMetadata, MediaPlayback, MediaPosition, SeekDirection};
use std::process::exit;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum AppScreen {
    Current,
    Songs,
    Search,
}

pub struct App {
    screen: AppScreen,
    songs: Vec<Song>,
    player_handle: PlayerHandle,
    media_controls: Option<MediaController>,
    selected_idx: Option<usize>,
    playing_idx: Option<usize>,
    cover_image: Option<TextureHandle>,
    default_cover_image: DynamicImage,
    config: Config,
    is_config_applied: bool, // We don't have `ui` in Default so we have to apply it on our first frame
    last_saved: Instant,
    screen_changed: bool,
}

impl App {
    fn draw(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if self.screen_changed {
            self.screen_changed = false; // Resetting it next frame after stuff was redrawn
        }
        Panel::bottom("player").min_size(70.0).show(ui, |ui| {
            self.draw_player(ui);
        });

        // Here self.screen_changed is set
        Panel::left("sidebar")
            .resizable(true)
            .default_size(150.0)
            .size_range(150.0..=250.0)
            .show(ui, |ui| {
                self.draw_sidebar(ui);
            });

        // Here it's used straight away
        CentralPanel::default().show(ui, |ui| match self.screen {
            AppScreen::Current => self.draw_central_current(ui, _frame),
            AppScreen::Songs => self.draw_central_songs(ui, _frame),
            AppScreen::Search => self.draw_central_search(ui, _frame),
        });

        // So we can safely reset self.screen_changed at the start of the next frame

        ui.request_repaint_after(Duration::from_millis(100));
    }

    fn draw_player(&mut self, ui: &mut Ui) {
        let mut p = self
            .player_handle
            .shared
            .position_ms
            .load(Ordering::Relaxed);
        if ui
            .scope(|ui| {
                ui.style_mut().spacing.slider_width = ui.available_width();
                ui.add(
                    Slider::new(
                        &mut p,
                        0..=self
                            .player_handle
                            .shared
                            .duration_ms
                            .load(Ordering::Relaxed),
                    )
                    .show_value(false)
                    .trailing_fill(true),
                )
            })
            .inner
            .drag_stopped()
        {
            self.seek(p);
        }

        let row_rect = ui.available_rect_before_wrap();

        center_horizontal(ui, |ui| {
            if circular_button(
                ui,
                50.0,
                RichText::new("🔀")
                    .size(35.0)
                    .color(if self.config.shuffle {
                        Color32::ORANGE
                    } else {
                        Color32::PLACEHOLDER
                    }),
            )
            .clicked()
            {
                // self.prev_song(ui);
                self.config.shuffle = !self.config.shuffle;
                self.save_config();
            }
            if circular_button(ui, 50.0, RichText::new("⏮️").size(35.0)).clicked() {
                self.prev_song(ui);
            }
            if circular_button(
                ui,
                50.0,
                RichText::new(if self.is_playing() {
                    "⏸️"
                } else {
                    "▶️"
                })
                .size(35.0),
            )
            .clicked()
            {
                self.toggle_pause();
            }
            if circular_button(ui, 50.0, RichText::new("⏭️").size(35.0)).clicked() {
                self.next_song(ui);
            }
            if circular_button(
                ui,
                50.0,
                RichText::new("🔂")
                    .size(35.0)
                    .color(if self.config.loop_one {
                        Color32::ORANGE
                    } else {
                        Color32::PLACEHOLDER
                    }),
            )
            .clicked()
            {
                self.config.loop_one = !self.config.loop_one;
                self.save_config();
            }
        });

        // 3. Define a fixed width for the volume slider UI
        let volume_ui_width = 150.0;

        // 4. Carve out a rectangle pinned to the absolute right side of the row
        let right_slider_rect = Rect::from_min_max(
            pos2(row_rect.max.x - volume_ui_width, row_rect.min.y),
            pos2(row_rect.max.x, row_rect.max.y),
        );

        let mut v = self.player_handle.shared.volume.lock().unwrap().clone();
        if ui
            .scope(|ui| {
                ui.style_mut().spacing.slider_width = 140.0;
                ui.put(
                    right_slider_rect,
                    Slider::new(&mut v, 0.0..=1.0)
                        .show_value(false)
                        .trailing_fill(true),
                )
            })
            .inner
            .changed()
        {
            self.set_volume(v);
        }
    }

    fn draw_sidebar(&mut self, ui: &mut Ui) {
        ui.take_available_width();
        ui.horizontal(|ui| {
            ui.add_sized(
                [ui.available_width(), 20.0],
                Label::new(RichText::new("Sections").heading()),
            );
        });
        let prev = self.screen;
        sidebar_entry(ui, &mut self.screen, AppScreen::Current, "Player");
        sidebar_entry(ui, &mut self.screen, AppScreen::Songs, "Songs");
        sidebar_entry(ui, &mut self.screen, AppScreen::Search, "Search");
        if self.screen != prev {
            self.screen_changed = true;
        }
    }

    // Central Panel modes
    fn draw_central_current(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if let Some(texture_handle) = &self.cover_image
            && let Some(idx) = self.playing_idx
        {
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

    fn draw_central_songs(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if self.screen_changed {
            self.selected_idx = self.playing_idx;
        }
        self.song_list(ui, "songs", |x| x.iter().enumerate().map(|x| x.0).collect());
    }

    fn draw_central_search(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.add(
            egui::TextEdit::singleline(&mut self.config.search_input)
                .desired_width(f32::INFINITY)
                .font(egui::FontId::proportional(32.0)),
        );

        let s = self.config.search_input.to_lowercase();
        self.song_list(ui, "search", |x| {
            x.iter()
                .enumerate()
                .filter(|(_, x)| x.display().to_lowercase().contains(&*s))
                .map(|(x, _)| x)
                .collect()
        });
    }

    fn song_list(&mut self, ui: &mut Ui, id: &'static str, f: impl FnOnce(&[Song]) -> Vec<usize>) {
        let indexes = f(&self.songs);
        let row_h = 40.0;
        let mut new_ind = None;
        let mut is_dc = false;

        let mut area = ScrollArea::vertical().id_salt(id).auto_shrink(false);
        if self.screen_changed {
            if let Some(row) = indexes.iter().position(|&i| Some(i) == self.selected_idx) {
                let step = row_h + ui.spacing().item_spacing.y; // show_rows uses row height + spacing
                let target = row as f32 * step - ui.available_height() / 2.0 + row_h / 2.0;
                area = area.vertical_scroll_offset(target.max(0.0));
            }
        }

        area.show_rows(ui, row_h, indexes.len(), |ui, row_range| {
            for ind in row_range {
                // let song = &songs[ind];
                let ind = indexes[ind];
                let song = &self.songs[ind];
                let resp = sized_entry(
                    ui,
                    40.0,
                    Align2::LEFT_TOP,
                    self.selected_idx == Some(ind),
                    &song.derived_name,
                );
                if resp.clicked() {
                    new_ind = Some(ind); // Just to make borrow checker shut the fuck up
                }
                if resp.double_clicked() {
                    is_dc = true;
                    self.screen = AppScreen::Current;
                }
            }
        });
        if let Some(ind) = new_ind {
            self.selected_idx = Some(ind);
        }
        if is_dc {
            self.playing_idx = new_ind;
            self.set_track(ui, new_ind);
            self.play_selected();
        }
    }

    fn set_track(&mut self, ui: &mut Ui, idx: Option<usize>) {
        self.playing_idx = idx;
        self.selected_idx = idx;
        if idx.is_none() {
            self.cover_image = None;
            return;
        }
        let idx = self.playing_idx.unwrap();
        let song = &self.songs[idx];
        let rgba = if let Some(art) = &song.get_art() {
            match image::load_from_memory(art) {
                Ok(s) => s.to_rgba8(),
                Err(_) => self.default_cover_image.to_rgba8(),
            }
        } else {
            self.default_cover_image.to_rgba8()
        };

        let size = [rgba.width() as usize, rgba.height() as usize];

        let color_image =
            ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
        let texture_handle = ui.load_texture("cover_art", color_image, TextureOptions::default());
        self.cover_image = Some(texture_handle);
    }

    fn change_track(&mut self, ui: &mut Ui, offset: i32) {
        // Calculate next index
        let mut idx = match self.playing_idx {
            Some(idx) => idx,
            None => return, // Moving on None doesn't do shit, so we don't care in the slightest
        };
        if offset < 0 && idx as i32 + offset < 0 {
            idx = (self.songs.len() as i32 + idx as i32 + offset) as usize;
        } else {
            idx = (idx as i32 + offset) as usize % self.songs.len();
        }

        self.set_track(ui, Some(idx));
    }

    fn randomize_track(&mut self, ui: &mut Ui) {
        self.set_track(ui, Some(random_range(0..self.songs.len())));
    }

    fn toggle_pause(&self) {
        if self.player_handle.shared.playing.load(Ordering::Relaxed) {
            self.pause();
        } else {
            self.resume();
        }
    }

    pub fn pause(&self) {
        self.player_handle.tx.send(Command::Pause).unwrap();
    }

    pub fn resume(&self) {
        self.player_handle.tx.send(Command::Resume).unwrap();
    }

    fn play_selected(&mut self) {
        match self.playing_idx {
            Some(idx) => {
                self.player_handle
                    .tx
                    .send(Command::Play(self.songs[idx].path.clone()))
                    .unwrap();
            }
            None => {
                self.player_handle.tx.send(Command::Stop).unwrap();
            }
        }
        self.save_config();
    }

    fn next_song(&mut self, ui: &mut Ui) {
        if self.config.shuffle {
            self.randomize_track(ui);
        } else {
            self.change_track(ui, 1);
        }
        self.play_selected();
    }

    fn prev_song(&mut self, ui: &mut Ui) {
        if self.config.shuffle {
            self.randomize_track(ui);
        } else {
            self.change_track(ui, -1);
        }
        self.play_selected();
    }

    fn seek(&mut self, seek_point_ms: u64) {
        self.player_handle
            .tx
            .send(Command::Seek(Duration::from_millis(seek_point_ms)))
            .unwrap();
        self.save_config();
    }

    fn set_volume(&mut self, volume: f32) {
        self.player_handle
            .tx
            .send(Command::SetVolume(volume))
            .unwrap();
        self.save_config();
    }

    fn save_config(&mut self) {
        if !self.is_config_applied {
            return; // Config isn't fully applied so saving at this point will fuck up the config state
        }
        self.config.song_id = self.playing_idx;
        self.config.volume = *self.player_handle.shared.volume.lock().unwrap();
        self.config.progress_ms = self
            .player_handle
            .shared
            .position_ms
            .load(Ordering::Relaxed);
        self.config.save();
        self.last_saved = Instant::now();
    }

    fn is_playing(&self) -> bool {
        self.player_handle.shared.playing.load(Ordering::Relaxed)
    }
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load_or_default();

        let s = Self {
            screen: AppScreen::Current,
            songs: vec![],
            selected_idx: None,
            playing_idx: None,
            cover_image: None,
            player_handle: spawn_player(),
            media_controls: start_souvlaki_thread(),
            config,
            is_config_applied: false,
            last_saved: Instant::now(),
            screen_changed: true,
            default_cover_image: image::load_from_memory(include_bytes!(
                "../assets/missing-draft.png"
            ))
            .unwrap(),
        };

        if s.media_controls.is_none() {
            println!("Failed to initialize media controls");
        }

        s
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.save_config();
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if !self.is_config_applied {
            self.songs = load_songs(self.config.dir.to_str().unwrap(), true).unwrap_or(vec![]);

            self.set_volume(self.config.volume);
            self.set_track(ui, self.config.song_id);
            self.play_selected();
            self.pause();
            self.seek(self.config.progress_ms);
            self.is_config_applied = true;
            self.save_config();
        }

        if !ui.ctx().egui_wants_keyboard_input() {
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                ui.send_viewport_cmd(ViewportCommand::Close);
            }
            if ui.input(|i| i.key_pressed(Key::Space)) {
                self.toggle_pause();
            }
            if ui.input(|i| i.key_pressed(Key::N)) {
                self.next_song(ui);
            }
            if ui.input(|i| i.key_pressed(Key::B)) {
                self.prev_song(ui);
            }
        }

        if self.last_saved.elapsed().as_secs() > 5 && self.is_playing() {
            self.save_config();
        }

        while let Ok(ev) = self.player_handle.events.try_recv() {
            match ev {
                Event::Finished => {
                    if self.config.loop_one {
                        self.play_selected();
                    } else {
                        self.next_song(ui);
                    }
                }
                Event::SongLoaded => {
                    if let Some(controls) = &mut self.media_controls
                        && let Some(playing_idx) = self.playing_idx
                    {
                        let song = &self.songs[playing_idx];
                        let s = MediaMetadata {
                            title: song.title.as_deref(),
                            album: song.album.as_deref(),
                            artist: song
                                .artists
                                .as_ref()
                                .and_then(|s| s.first().map(|a| a.as_str())),
                            duration: Some(Duration::from_millis(
                                self.player_handle
                                    .shared
                                    .duration_ms
                                    .load(Ordering::Relaxed),
                            )),
                            cover_url: None,
                        };

                        let _ = controls.media_controls.set_metadata(s);
                    }
                }
            }
        }

        let mut events = vec![]; // Fighting borrow checker because... yikes
        if let Some(controls) = &mut self.media_controls {
            while let Ok(ev) = controls.events.try_recv() {
                events.push(ev);
            }

            let _ = controls
                .media_controls
                .set_volume(*self.player_handle.shared.volume.lock().unwrap() as f64);

            let _ = controls.media_controls.set_playback(
                if !self
                    .player_handle
                    .shared
                    .track_loaded
                    .load(Ordering::Relaxed)
                {
                    MediaPlayback::Stopped
                } else if !self.player_handle.shared.playing.load(Ordering::Relaxed) {
                    MediaPlayback::Paused {
                        progress: Some(MediaPosition(Duration::from_millis(
                            self.player_handle
                                .shared
                                .position_ms
                                .load(Ordering::Relaxed),
                        ))),
                    }
                } else {
                    MediaPlayback::Playing {
                        progress: Some(MediaPosition(Duration::from_millis(
                            self.player_handle
                                .shared
                                .position_ms
                                .load(Ordering::Relaxed),
                        ))),
                    }
                },
            );
        }

        for ev in events {
            match ev {
                MediaControlEvent::Play => self.resume(),
                MediaControlEvent::Pause => self.pause(),
                MediaControlEvent::Toggle => self.toggle_pause(),
                MediaControlEvent::Next => self.next_song(ui),
                MediaControlEvent::Previous => self.prev_song(ui),
                MediaControlEvent::Stop => {
                    self.playing_idx = None;
                    self.play_selected()
                }
                MediaControlEvent::Seek(_) => {
                    println!(
                        "IT ACTUALLY HAPPENED! WE GOT MediaControlEvent::Seek LET'S GOOOO. What kind of loser even tried to call this LMAO?"
                    )
                } // Sounds like some kind of bs. I'll set as todo and figure it out later if needed
                MediaControlEvent::SeekBy(sd, d) => {
                    // Gonna give this thing a d omegalul
                    match sd {
                        SeekDirection::Forward => {
                            let current = self
                                .player_handle
                                .shared
                                .position_ms
                                .load(Ordering::Relaxed);
                            self.seek(current + d.as_millis() as u64);
                        }
                        SeekDirection::Backward => {
                            let current = self
                                .player_handle
                                .shared
                                .position_ms
                                .load(Ordering::Relaxed);
                            let offset = (d.as_millis() as u64).min(current);
                            self.seek(current - offset);
                        }
                    }
                }
                MediaControlEvent::SetPosition(pos) => {
                    self.seek(pos.0.as_millis() as u64);
                }
                MediaControlEvent::SetVolume(v) => {
                    self.set_volume(v as f32);
                }
                MediaControlEvent::SetShuffle(_) => {} // Not implemented :D
                MediaControlEvent::OpenUri(_) => {}    // Not gonna happen
                MediaControlEvent::Raise => ui.send_viewport_cmd(ViewportCommand::Focus),
                MediaControlEvent::Quit => exit(0), // Uh... you sure this should be an option...? Fine ig...
                e => {
                    println!(
                        "oOoOh, a lib update? Yeah, no idea what to do with {:?} lol",
                        e
                    );
                }
            }
        }

        self.draw(ui, _frame);
    }
}
