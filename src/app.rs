use egui::{Vec2, epaint::text::{FontInsert, InsertFontFamily}};

use crate::models::audio_file::AudioFile;

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
enum FileFormat {
    FLAC,
    MP3,
    AAC,
    OPUS,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AudioConverterApp {
    #[serde(skip)]
    files: Vec<AudioFile>,

    // Interaction
    table_selection: Option<usize>,

    // Settings
    out_format: FileFormat,
    out_bitrate: u64,
    out_directory: String,
}

impl Default for AudioConverterApp {
    fn default() -> Self {
        Self {
            files: Vec::new(),

            table_selection: None,

            out_format: FileFormat::OPUS,
            out_bitrate: 128000,
            out_directory: "./".to_string(),
        }
    }
}

// Empty placeholder texts
const NO_ARTIST: &str = "[no artist]";
const NO_ALBUM: &str = "";
const NO_TITLE: &str = "Untitled";

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

        Default::default()
    }

    fn preview_dropped_files(&mut self, ctx: &egui::Context) {
        use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
        use std::fmt::Write as _;

        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                let mut text = "Adding files to queue:\n".to_owned();
                for file in &i.raw.hovered_files {
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

        ctx.input(|i| {
            // if !i.raw.hovered_files.is_empty() {
            //     println!("hovering a file: {}", i.raw.hovered_files.len());
            // }

            if !i.raw.dropped_files.is_empty() {
                // println!("dropped files");
                for file in &i.raw.dropped_files {
                    self.files.push(AudioFile::new(file.clone().path.unwrap()));
                }
            }
        });
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

                for (i, file) in self.files.iter().enumerate() {
                    body.row(text_height, |mut row| {
                        row.set_selected(self.table_selection == Some(i));

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
                            clicked_row = Some(i);
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
        egui::Grid::new("format_settings")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.heading("Format Settings");
                ui.end_row();

                ui.label("Audio codec");
                egui::ComboBox::from_id_salt("output_format_combobox")
                    .selected_text(match self.out_format {
                        FileFormat::FLAC => ".flac",
                        FileFormat::MP3 => ".mp3",
                        FileFormat::AAC => ".aac",
                        FileFormat::OPUS => ".opus",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.out_format, FileFormat::FLAC, ".flac");
                        ui.selectable_value(&mut self.out_format, FileFormat::MP3, ".mp3");
                        ui.selectable_value(&mut self.out_format, FileFormat::AAC, ".aac");
                        ui.selectable_value(&mut self.out_format, FileFormat::OPUS, ".opus");
                    });
                ui.end_row();

                ui.label("Bitrate");
                ui.add(egui::DragValue::new(&mut self.out_bitrate).speed(1000.0));
                ui.end_row();
            });

        ui.separator();

        egui::Grid::new("output_settings")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.heading("Output settings");
                ui.end_row();

                ui.label("Output Directory");
                ui.text_edit_singleline(&mut self.out_directory);
            });

        ui.separator();

        if ui.button("Convert!").clicked() {}
    }

    fn file_info_popup(&mut self, ctx: &egui::Context) {
        use egui::Align2;

        let file = self.files.get(self.table_selection.unwrap()).unwrap();

        egui::Window::new("File information")
            .movable(true)
            .min_width(300.0)
            .max_width(300.0)
            .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
            .resizable(false)
            .show(ctx, |ui| {
                // TODO: maybe not clone
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

                let texture = file.load_album_art(ctx);

                if let Some(texture) = texture {
                    ui.add(
                        egui::Image::from_texture(&texture)
                            .fit_to_fraction(Vec2::ONE)
                            .max_width(300.0)
                            .corner_radius(5),
                    );
                } else {
                    ui.label("<No image>");
                }
            });
    }
}

impl eframe::App for AudioConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("Batch Audio File Converter");
        });

        egui::SidePanel::right("output_settings").show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            self.settings_list(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Files");

                ui.add_space(10.0);

                if ui.button("Open files").clicked()
                    && let Some(paths) = rfd::FileDialog::new().pick_files()
                {
                    for file in paths {
                        self.files.push(AudioFile::new(file));
                    }
                }
            });

            egui::ScrollArea::horizontal().show(ui, |ui| {
                self.file_table(ui);
            });

            if self.files.is_empty() {
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
    }
}
