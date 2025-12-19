use crate::models::AudioFile;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AudioConverterApp {
    files: Vec<AudioFile>,
}

impl Default for AudioConverterApp {
    fn default() -> Self {
        Self { files: Vec::new() }
    }
}

impl AudioConverterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
                    self.files
                        .push(AudioFile::new_from_dropped_file(file.clone()));
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
            .column(Column::auto())
            .column(
                Column::remainder()
                    .at_least(50.0)
                    .clip(true)
                    .resizable(true),
            )
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        table = table.sense(egui::Sense::click());

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Artist");
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
                            ui.label(file.artist.as_deref().unwrap_or("No artist"));
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
            ui.heading("Files");

            if ui.button("Open file").clicked() {}

            egui::ScrollArea::vertical().show(ui, |ui| {
                self.file_table(ui);
            });
        });

        egui::SidePanel::right("output_settings").show(ctx, |ui| {
            ui.heading("Settings");

            ui.label("Format Settings");

            ui.separator();
            ui.label("other settings idk");
        });

        self.preview_dropped_files(ctx);
    }
}
