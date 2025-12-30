use egui::{
    Vec2,
    epaint::text::{FontInsert, InsertFontFamily},
};
use std::sync::mpsc;

use crate::{
    models::audio_file::{AlbumArtError, AudioCodec, AudioContainer, AudioFile, get_image_hash},
    tasks_manager::TasksManager,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub enum OutputGrouping {
    NoGrouping,
    Copy,
    ArtistAlbum,
    Album,
    Artist,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone)]
enum AppTheme {
    System,
    Dark,
    Light,
}

pub const NO_ARTIST: &str = "<no artist>";
pub const NO_ALBUM: &str = "<no album>";
pub const NO_TITLE: &str = "<no title>";

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Settings {
    pub app_theme: AppTheme,

    pub run_concurrent_task_count: usize,

    pub out_codec: AudioCodec,
    pub out_container: AudioContainer,
    pub out_bitrate: usize,
    pub out_directory: String,
    pub out_grouping: OutputGrouping,
    pub out_embed_art: bool,
}

pub struct AppState {
    files: Vec<AudioFile>,
    album_art_rx: Option<mpsc::Receiver<Result<egui::ColorImage, AlbumArtError>>>,
    album_art_hash: Option<u64>,
    album_art: Option<egui::TextureHandle>,
    prev_album_art_path: Option<std::path::PathBuf>,

    is_transcoding: bool,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AudioConverterApp {
    #[serde(skip)]
    app_state: AppState,
    #[serde(skip)]
    tasks_manager: TasksManager,

    // Interaction
    #[serde(skip)]
    table_selection: Option<usize>,

    pub settings: Settings,
}

impl Default for AudioConverterApp {
    fn default() -> Self {
        Self {
            app_state: AppState {
                files: Vec::new(),
                album_art_rx: None,
                album_art_hash: None,
                album_art: None,
                prev_album_art_path: None,
                is_transcoding: false,
            },
            tasks_manager: TasksManager::new(),

            table_selection: None,

            settings: Settings {
                app_theme: AppTheme::System,
                run_concurrent_task_count: 2,
                out_codec: AudioCodec::OPUS,
                out_container: AudioContainer::OGG,
                out_bitrate: 64000,
                out_directory: "./".to_string(),
                out_grouping: OutputGrouping::ArtistAlbum,
                out_embed_art: true,
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
            .body(|mut body| {
                let mut clicked_row: Option<usize> = None;

                for file in &self.app_state.files {
                    body.row(text_height, |mut row| {
                        row.set_selected(self.table_selection == Some(row.index()));

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
                    });
                }

                if let Some(i) = clicked_row {
                    self.toggle_row_selection(i);
                }
            });
    }

    fn toggle_row_selection(&mut self, row_index: usize) {
        if self.table_selection == Some(row_index) {
            self.table_selection = None;
        } else {
            self.table_selection = Some(row_index);
        }
    }

    fn settings_list(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("settings")
            .num_columns(2)
            // .striped(true)
            .show(ui, |ui| {
                ui.label("Theme");
                egui::ComboBox::from_id_salt("app_theme")
                    .selected_text(match self.settings.app_theme {
                        AppTheme::System => "Follow system",
                        AppTheme::Dark => "Dark",
                        AppTheme::Light => "Light",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.settings.app_theme, AppTheme::System, "Follow system");
                        ui.selectable_value(&mut self.settings.app_theme, AppTheme::Dark, "Dark");
                        ui.selectable_value(&mut self.settings.app_theme, AppTheme::Light, "Light");
                    });
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.end_row();

                ui.heading("Runtime Settings");
                ui.end_row();

                ui.label("Concurrent tasks");
                ui.add(
                    egui::DragValue::new(&mut self.settings.run_concurrent_task_count)
                        .fixed_decimals(0)
                        .speed(1.0)
                        .range(1..=10),
                );
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.end_row();

                ui.heading("Output settings");
                ui.end_row();

                ui.label("Audio codec");
                egui::ComboBox::from_id_salt("output_codec_combobox")
                    .selected_text(match self.settings.out_codec {
                        AudioCodec::FLAC => "FLAC",
                        AudioCodec::MP3 => "MP3",
                        AudioCodec::AAC => "AAC",
                        AudioCodec::OPUS => "OPUS",
                        AudioCodec::VORBIS => "VORBIS",
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(
                                &mut self.settings.out_codec,
                                AudioCodec::FLAC,
                                "FLAC",
                            )
                            .clicked()
                        {
                            self.settings.out_container = AudioContainer::FLAC;
                        }
                        if ui
                            .selectable_value(&mut self.settings.out_codec, AudioCodec::MP3, "MP3")
                            .clicked()
                        {
                            self.settings.out_container = AudioContainer::MP3;
                        }
                        if ui
                            .selectable_value(&mut self.settings.out_codec, AudioCodec::AAC, "AAC")
                            .clicked()
                        {
                            self.settings.out_container = AudioContainer::M4A;
                        }
                        if ui
                            .selectable_value(
                                &mut self.settings.out_codec,
                                AudioCodec::OPUS,
                                "OPUS",
                            )
                            .clicked()
                        {
                            self.settings.out_container = AudioContainer::OGG;
                        }
                        if ui
                            .selectable_value(
                                &mut self.settings.out_codec,
                                AudioCodec::VORBIS,
                                "VORBIS",
                            )
                            .clicked()
                        {
                            self.settings.out_container = AudioContainer::OGG;
                        };
                    });
                ui.end_row();

                ui.label("Audio container");
                egui::ComboBox::from_id_salt("output_container_combobox")
                    .selected_text(match self.settings.out_container {
                        AudioContainer::FLAC => ".flac",
                        AudioContainer::MP3 => ".mp3",
                        AudioContainer::M4A => ".m4a",
                        AudioContainer::OGG => ".ogg",
                        AudioContainer::OPUS => ".opus",
                    })
                    .show_ui(ui, |ui| match self.settings.out_codec {
                        AudioCodec::FLAC => {
                            ui.selectable_value(
                                &mut self.settings.out_container,
                                AudioContainer::FLAC,
                                ".flac",
                            );
                        }
                        AudioCodec::MP3 => {
                            ui.selectable_value(
                                &mut self.settings.out_container,
                                AudioContainer::MP3,
                                ".mp3",
                            );
                        }
                        AudioCodec::AAC => {
                            ui.selectable_value(
                                &mut self.settings.out_container,
                                AudioContainer::M4A,
                                ".m4a",
                            );
                        }
                        AudioCodec::OPUS => {
                            ui.selectable_value(
                                &mut self.settings.out_container,
                                AudioContainer::OPUS,
                                ".opus",
                            );
                            ui.selectable_value(
                                &mut self.settings.out_container,
                                AudioContainer::OGG,
                                ".ogg",
                            );
                        }
                        AudioCodec::VORBIS => {
                            ui.selectable_value(
                                &mut self.settings.out_container,
                                AudioContainer::OGG,
                                ".ogg",
                            );
                        }
                    });
                ui.end_row();

                ui.label("Bitrate");
                ui.add(
                    egui::DragValue::new(&mut self.settings.out_bitrate)
                        .fixed_decimals(0)
                        .speed(1000.0),
                );
                ui.end_row();

                let text_width = ui.available_width().min(240.0);

                ui.label("Output Directory");
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [text_width, ui.text_style_height(&egui::TextStyle::Body)],
                            egui::TextEdit::singleline(&mut self.settings.out_directory),
                        )
                        .double_clicked()
                        || ui.button("🗁").clicked()
                    {
                        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                            self.settings.out_directory = dir.to_str().unwrap().to_string();
                        }
                    }
                });
                ui.end_row();

                ui.label("Group by...")
                    .on_hover_text_at_pointer("Group output files in a folder");
                egui::ComboBox::from_id_salt("output_grouping_combobox")
                    .selected_text(match self.settings.out_grouping {
                        OutputGrouping::NoGrouping => "No Grouping",
                        OutputGrouping::Copy => "Copy from source",
                        OutputGrouping::ArtistAlbum => "Artist - Album",
                        OutputGrouping::Album => "Album",
                        OutputGrouping::Artist => "Artist",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.settings.out_grouping,
                            OutputGrouping::NoGrouping,
                            "No Grouping",
                        )
                        .on_hover_text_at_pointer(
                            "Group output files in a folder:\n - No grouping/folders",
                        );
                        ui.selectable_value(
                            &mut self.settings.out_grouping,
                            OutputGrouping::Copy,
                            "Copy from source",
                        )
                        .on_hover_text_at_pointer(
                            "Group output files in a folder:\n - Parent folder from original files",
                        );
                        ui.selectable_value(
                            &mut self.settings.out_grouping,
                            OutputGrouping::ArtistAlbum,
                            "Artist - Album",
                        )
                        .on_hover_text_at_pointer({
                            let first_file = self.app_state.files.first();
                            let artist = first_file.and_then(|f| f.artist.as_deref()).unwrap_or("Artist");
                            let album = first_file.and_then(|f| f.album.as_deref()).unwrap_or("Album");
                            format!("Group output files in a folder:\n - Create a folder name '{artist} - {album}'")
                        });
                        ui.selectable_value(
                            &mut self.settings.out_grouping,
                            OutputGrouping::Album,
                            "Album",
                        )
                        .on_hover_text_at_pointer({
                            let first_file = self.app_state.files.first();
                            let album = first_file.and_then(|f| f.album.as_deref()).unwrap_or("Album");
                            format!("Group output files in a folder:\n - Create a folder name '{album}'")
                        });
                        ui.selectable_value(
                            &mut self.settings.out_grouping,
                            OutputGrouping::Artist,
                            "Artist",
                        ).on_hover_text_at_pointer({
                            let first_file = self.app_state.files.first();
                            let artist = first_file.and_then(|f| f.artist.as_deref()).unwrap_or("Artist");
                            format!("Group output files in a folder:\n - Create a folder name '{artist}'")
                        });
                    }).response.on_hover_text_at_pointer("Group output files in a folder");
                ui.end_row();

                let cover_art_tooltip = "Toggle embedding cover art as a Vorbis metadata block\n - depending on the source file, it may inflate file size";
                ui.label("Embed cover art").on_hover_text_at_pointer(cover_art_tooltip);
                ui.checkbox(&mut self.settings.out_embed_art, "").on_hover_text_at_pointer(cover_art_tooltip);
                ui.end_row();
            });

        ui.separator();

        if ui.button("Convert!").clicked() {
            for file in &self.app_state.files {
                self.tasks_manager.queue_audio_file(file.clone());
                self.app_state.is_transcoding = true;
            }
        }
    }

    fn task_queue_window(&mut self, ctx: &egui::Context) {
        use egui::Align2;

        egui::Window::new("Task Queue")
            .anchor(Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0))
            .movable(false)
            .resizable(false)
            .title_bar(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Queue");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spinner();
                    })
                });
                ui.label(format!(
                    "Tasks remaining: {}",
                    self.tasks_manager.queue.len() + self.tasks_manager.active_tasks.len()
                ));
                ui.separator();

                for task in &self.tasks_manager.active_tasks {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} - {} on {}",
                            task.file.artist.clone().unwrap_or(NO_ARTIST.to_string()),
                            task.file.title.clone().unwrap_or(NO_TITLE.to_string()),
                            task.file.album.clone().unwrap_or(NO_ALBUM.to_string())
                        ));
                    });
                }
            });
    }

    fn file_info_popup(&mut self, ctx: &egui::Context) {
        use egui::Align2;

        let file = self
            .app_state
            .files
            .get(self.table_selection.unwrap())
            .unwrap();

        egui::Window::new("File information")
            .min_width(300.0)
            .max_width(300.0)
            .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
            .resizable(false)
            .movable(false)
            .default_open(false)
            .show(ctx, |ui| {
                ui.heading(file.title.clone().unwrap_or(NO_TITLE.to_string()));
                egui::Grid::new("detailed_file_info")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Artist:");
                        ui.label(file.artist.clone().unwrap_or(NO_ARTIST.to_string()));
                        ui.end_row();

                        ui.label("Album:");
                        ui.label(file.album.clone().unwrap_or(NO_ALBUM.to_string()));
                        ui.end_row();

                        ui.label("File path:");
                        ui.add(
                            egui::Label::new(file.path.clone().to_string_lossy().to_string())
                                .wrap(),
                        );
                        ui.end_row();
                    });

                ui.separator();

                // NOTE: This entire bit is so scuffed, needs a rewrite
                let parent_changed = self
                    .app_state
                    .prev_album_art_path
                    .as_ref()
                    .map(|p| p.parent())
                    != Some(file.path.parent());
                let needs_reload = self.app_state.album_art_rx.is_none()
                    && (self.app_state.album_art.is_none() || parent_changed); // TODO: check hashes
                if needs_reload {
                    self.app_state.prev_album_art_path = Some(file.path.clone());
                    self.app_state.album_art_rx = Some(file.load_album_art());
                    self.app_state.album_art = None;
                }

                if let Some(rx) = &self.app_state.album_art_rx {
                    match rx.try_recv() {
                        Ok(Ok(image)) => {
                            let hash = get_image_hash(image.as_raw());

                            if self.app_state.album_art_hash == Some(hash) {
                                self.app_state.album_art_rx = None;
                                return;
                            }

                            let texture = ctx.load_texture(
                                format!("album_{}", hash),
                                image,
                                egui::TextureOptions::LINEAR,
                            );

                            self.app_state.album_art_hash = Some(hash);
                            self.app_state.album_art = Some(texture);
                            self.app_state.album_art_rx = None;

                            ctx.request_repaint();
                        }
                        Ok(Err(_)) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            self.app_state.album_art_rx = None;
                        }
                        Err(mpsc::TryRecvError::Empty) => {
                            let _ = ui.label("Loading image...");
                        }
                    }
                }

                if let Some(texture) = &self.app_state.album_art {
                    ui.add(
                        egui::Image::from_texture(texture)
                            .fit_to_fraction(Vec2::ONE)
                            .max_width(300.0)
                            .corner_radius(5),
                    );
                }
            });
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
                self.settings_list(ui);
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
                        let audio_file = match AudioFile::new(file) {
                            Ok(af) => af,
                            Err(_) => continue,
                        };

                        self.app_state.files.push(audio_file);
                    }
                }

                if ui.button("Open folders").clicked()
                    && let Some(paths) = rfd::FileDialog::new()
                        .pick_folders()
                {
                    for directory in &paths {
                        let mut files = match AudioFile::from_directory(directory) {
                            Ok(f) => f,
                            Err(_) => continue,
                        };
                        self.app_state.files.append(&mut files);
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

        if self.table_selection.is_some() {
            self.file_info_popup(ctx);
        }

        self.preview_dropped_files(ctx);
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        if path.is_dir() {
                            let mut files = match AudioFile::from_directory(path) {
                                Ok(f) => f,
                                Err(_) => continue, // TODO: maybe consider actually error handling
                            };
                            self.app_state.files.append(&mut files);
                        } else {
                            self.app_state
                                .files
                                .push(AudioFile::new(path.clone()).unwrap()); // TODO: error handle
                        }
                    }
                }
            }
        });

        if self.app_state.is_transcoding {
            self.task_queue_window(ctx);
        }
    }
}
