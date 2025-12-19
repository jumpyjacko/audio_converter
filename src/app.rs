use egui::epaint::text::{FontInsert, InsertFontFamily};

use crate::models::audio_file::AudioFile;

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
enum FileFormat {
    FLAC,
    MP3,
    AAC,
    OPUS,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AudioConverterApp {
    files: Vec<AudioFile>,
    out_format: FileFormat,
    out_bitrate: u64,
    out_directory: String,
}

impl Default for AudioConverterApp {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            out_format: FileFormat::OPUS,
            out_bitrate: 128000,
            out_directory: "./".to_string(),
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
            .column(Column::auto().at_least(100.0).resizable(true))
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
                for file in &self.files {
                    body.row(text_height, |mut row| {
                        row.col(|ui| {
                            ui.label(file.track.as_deref().unwrap_or(""));
                        });
                        row.col(|ui| {
                            ui.label(file.artist.as_deref().unwrap_or("No artist"));
                        });
                        row.col(|ui| {
                            ui.label(file.album.as_deref().unwrap_or(""));
                        });
                        row.col(|ui| {
                            ui.label(file.title.as_deref().unwrap_or("Untitled"));
                        });
                        row.col(|ui| {
                            ui.label(file.path.to_string_lossy());
                        });
                    })
                }
            });
    }
}

impl eframe::App for AudioConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("Batch Audio File Converter");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Files");

                ui.add_space(10.0);

                if ui.button("Open file").clicked()
                    && let Some(paths) = rfd::FileDialog::new().pick_files()
                {
                    for file in paths {
                        self.files.push(AudioFile::new(file));
                    }
                }
            });

            self.file_table(ui);
        });

        egui::SidePanel::right("output_settings").show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

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
        });

        self.preview_dropped_files(ctx);
    }
}
