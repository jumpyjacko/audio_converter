use egui::{
    Key, Modifiers,
    epaint::text::{FontInsert, InsertFontFamily},
};
use std::{collections::HashSet, sync::mpsc};

use crate::models::audio_file::{
    AlbumArtError, AudioCodec, AudioContainer, AudioFile, AudioSampleRate,
};
use crate::models::settings::{AppTheme, OutputGrouping, Settings};
use crate::tasks_manager::TasksManager;
use crate::ui;

pub const NO_ARTIST: &str = "<no artist>";
pub const NO_ALBUM: &str = "<no album>";
pub const NO_TITLE: &str = "<no title>";

pub struct AppState {
    pub files: Vec<AudioFile>,
    pub cover_art_rx: Option<mpsc::Receiver<Result<egui::ColorImage, AlbumArtError>>>,
    pub cover_art: Option<egui::TextureHandle>,

    pub lg_cover_art_rx: Option<mpsc::Receiver<Result<egui::ColorImage, AlbumArtError>>>,
    pub lg_cover_art: Option<egui::TextureHandle>,
    pub showing_lg_art: bool,

    pub is_transcoding: bool,

    pub table_selections: HashSet<usize>,
    pub first_selection: Option<usize>,
    pub last_selection: Option<usize>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AudioConverterApp {
    pub settings: Settings,

    #[serde(skip)]
    tasks_manager: TasksManager,
    #[serde(skip)]
    app_state: AppState,
}

impl Default for AudioConverterApp {
    fn default() -> Self {
        Self {
            app_state: AppState {
                files: Vec::new(),
                cover_art_rx: None,
                cover_art: None,
                lg_cover_art_rx: None,
                lg_cover_art: None,
                showing_lg_art: false,
                is_transcoding: false,
                table_selections: HashSet::new(),
                first_selection: None,
                last_selection: None,
            },
            tasks_manager: TasksManager::new(),

            settings: Settings {
                app_theme: AppTheme::System,
                run_concurrent_task_count: 2,
                out_codec: AudioCodec::OPUS,
                out_container: AudioContainer::OGG,
                out_sample_rate: AudioSampleRate::Studio48,
                out_bitrate: 64000,
                out_directory: "./".to_string(),
                out_grouping: OutputGrouping::ArtistAlbum,
                out_embed_art: true,
                out_enable_cover_art_resize: false,
                out_cover_art_resolution: 1000,
            },
        }
    }
}

impl AudioConverterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.add_font(FontInsert::new(
            "Noto-Sans-CJK_SC",
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansCJKsc-Regular.otf"
            )),
            vec![InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Lowest,
            }],
        ));
        cc.egui_ctx.add_font(FontInsert::new(
            "Noto-Sans-CJK_JP",
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansCJKjp-Regular.otf"
            )),
            vec![InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Lowest,
            }],
        ));
        cc.egui_ctx.add_font(FontInsert::new(
            "Noto-Sans-CJK_KR",
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansCJKkr-Regular.otf"
            )),
            vec![InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Lowest,
            }],
        ));

        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn preview_dropped_files(&mut self, ctx: &egui::Context) {
        use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
        use std::fmt::Write as _;

        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                let mut text = "Adding files to queue:\n".to_owned();
                for (idx, file) in i.raw.hovered_files.iter().enumerate() {
                    if idx == 3 {
                        let _ = write!(text, "\n...\n{} more files", i.raw.hovered_files.len() - 3);
                        break;
                    }

                    if let Some(path) = &file.path {
                        write!(text, "\n{}", path.display()).ok();
                    } else if !file.mime.is_empty() {
                        write!(text, "\n{}", file.mime).ok();
                    } else {
                        text += "\n???";
                    }
                }
                text
            });

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let content_rect = ctx.content_rect();
            painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }
    }

    fn file_table(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let available_height = ui.available_height();
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto().at_least(75.0).resizable(true))
            .column(Column::auto().at_least(75.0).resizable(true))
            .column(Column::auto().at_least(75.0).resizable(true))
            .column(Column::remainder().resizable(true))
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        table = table.sense(egui::Sense::click());
        let mut clicked_row: Option<usize> = None;

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Track #");
                });
                header.col(|ui| {
                    ui.strong("Artist");
                });
                header.col(|ui| {
                    ui.strong("Album");
                });
                header.col(|ui| {
                    ui.strong("Song Title");
                });
                header.col(|ui| {
                    ui.strong("File Path");
                });
            })
            .body(|body| {
                let files = &self.app_state.files;
                let row_height = text_height;
                let num_rows = files.len();
                body.rows(row_height, num_rows, |mut row| {
                    if let Some(file) = files.get(row.index()) {
                        row.set_selected(self.app_state.table_selections.contains(&row.index()));

                        row.col(|ui| {
                            ui.label(file.track.as_deref().unwrap_or(""));
                        });
                        row.col(|ui| {
                            ui.label(file.artist.as_deref().unwrap_or(NO_ARTIST));
                        });
                        row.col(|ui| {
                            ui.label(file.album.as_deref().unwrap_or(NO_ALBUM));
                        });
                        row.col(|ui| {
                            ui.label(file.title.as_deref().unwrap_or(NO_TITLE));
                        });
                        row.col(|ui| {
                            ui.label(file.path.to_string_lossy());
                        });

                        if row.response().clicked() {
                            clicked_row = Some(row.index());
                        }
                    }
                });
            });

        // NOTE: this mess is for multi-select
        if let Some(i) = clicked_row {
            if self.app_state.first_selection.is_none() {
                self.app_state.first_selection = Some(i);
            }
            self.app_state.last_selection = Some(i);
            ui.input(|input| {
                if input.modifiers.command {
                    if self.app_state.table_selections.contains(&i) {
                        self.app_state.table_selections.remove(&i);
                    } else {
                        self.app_state.table_selections.insert(i);
                    }
                } else if self.app_state.first_selection.is_some() && input.modifiers.shift {
                    self.app_state.table_selections.clear();
                    if let Some(start) = self.app_state.first_selection {
                        self.app_state
                            .table_selections
                            .extend(start.min(i)..=start.max(i));
                    }
                } else {
                    if self.app_state.table_selections.len() == 1
                        && self.app_state.table_selections.contains(&i)
                    {
                        self.app_state.table_selections.clear();
                    } else {
                        self.app_state.table_selections.clear();
                        self.app_state.table_selections.insert(i);
                        self.app_state.first_selection = Some(i);
                    }
                }

                if self.app_state.table_selections.is_empty() {
                    self.app_state.first_selection = None;
                    self.app_state.last_selection = None;
                }
            });
            self.app_state.cover_art_rx = Some(self.app_state.files[i].load_album_art(Some(300)));
        }
    }
}

impl eframe::App for AudioConverterApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tasks_manager.update(&self.settings);
        self.app_state.is_transcoding = !self.tasks_manager.active_tasks.is_empty();

        match self.settings.app_theme {
            AppTheme::System => ctx.set_visuals(egui::Visuals::default()),
            AppTheme::Dark => ctx.set_visuals(egui::Visuals::dark()),
            AppTheme::Light => ctx.set_visuals(egui::Visuals::light()),
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("Batch Audio File Converter");
        });

        egui::SidePanel::right("output_settings").show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui::settings::settings_list(&mut self.settings, &self.app_state, ui);

                ui.separator();

                if ui.button("Convert!").clicked() {
                    for file in &self.app_state.files {
                        self.tasks_manager.queue_audio_file(file.clone());
                        self.app_state.is_transcoding = true;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Files");

                ui.add_space(10.0);

                if ui.button("Open files").clicked()
                    && let Some(paths) = rfd::FileDialog::new()
                        .add_filter("audio", &crate::models::audio_file::ALLOWED_INPUT_TYPES)
                        .pick_files()
                {
                    for file in paths {
                        let audio_file = AudioFile::new(file).unwrap();
                        self.app_state.files.push(audio_file);
                    }
                }

                if ui.button("Open folders").clicked()
                    && let Some(paths) = rfd::FileDialog::new()
                        .pick_folders()
                {
                    for directory in &paths {
                        let mut files = AudioFile::from_directory(directory).unwrap();
                        self.app_state.files.append(&mut files);
                    }
                }

                if !self.app_state.files.is_empty() {
                    if ui.button("Clear all").clicked()
                    {
                        self.app_state.files.clear();
                        self.app_state.table_selections.clear();
                        self.app_state.first_selection = None;
                        self.app_state.last_selection = None;
                    }
                }
            });

            egui::ScrollArea::horizontal().show(ui, |ui| {
                self.file_table(ui);
            });

            if self.app_state.files.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 20.0);
                    ui.heading("Drag and drop a file or folder into the window to get started or click the 'Open files' button");
                });
            }
        });

        if !self.app_state.table_selections.is_empty() {
            ui::file_info::file_info_popup(&mut self.app_state, ctx);

            if self.app_state.showing_lg_art {
                egui::Area::new(egui::Id::new("viewer"))
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        let _ = ui.allocate_rect(ctx.content_rect(), egui::Sense::click_and_drag());
                        ui::album_art_viewer::large_album_art_viewer(&mut self.app_state, ctx);
                    });
            }
        }

        self.preview_dropped_files(ctx);
        ctx.input(|i| {
            if i.raw.dropped_files.is_empty() {
                return;
            }
            for file in &i.raw.dropped_files {
                let Some(path) = &file.path else { continue };
                if path.is_dir() {
                    let mut files = AudioFile::from_directory(path).unwrap();
                    self.app_state.files.append(&mut files);
                } else {
                    self.app_state
                        .files
                        .push(AudioFile::new(path.clone()).unwrap());
                }
            }
        });

        if self.app_state.is_transcoding {
            ui::task_queue::task_queue_window(&mut self.tasks_manager, ctx);
        }

        ctx.input_mut(|input| {
            if input.key_pressed(Key::Delete) {
                if !self.app_state.table_selections.is_empty() {
                    self.app_state.files = self
                        .app_state
                        .files
                        .clone()
                        .into_iter()
                        .enumerate()
                        .filter(|(i, _)| !self.app_state.table_selections.contains(i))
                        .map(|(_, f)| f)
                        .collect();

                    self.app_state.table_selections.clear();
                    self.app_state.first_selection = None;
                    self.app_state.last_selection = None;
                }
            }

            // select all
            if !self.app_state.files.is_empty() {
                if input.consume_key(Modifiers::CTRL, Key::A) {
                    self.app_state.table_selections.clear();
                    self.app_state
                        .table_selections
                        .extend(0..self.app_state.files.len());
                    self.app_state.first_selection = Some(0);
                    self.app_state.last_selection = Some(0);
                    self.app_state.cover_art_rx =
                        Some(self.app_state.files[0].load_album_art(Some(300))); // refresh cover art
                }
            }
        });
    }
}
