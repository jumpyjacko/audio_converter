#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AudioConverterApp {
    files: Vec<String>,
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
                for _ in 0..10 {
                    body.row(text_height, |mut row| {
                        row.col(|ui| {
                            ui.label("Artist");
                        });
                        row.col(|ui| {
                            ui.label("Song title");
                        });
                        row.col(|ui| {
                            ui.label("/path/to/file");
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
    }
}
