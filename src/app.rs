#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AudioConverterApp {
    files: Vec<String>,
}

impl Default for AudioConverterApp {
    fn default() -> Self {
        Self {
            files: Vec::new(),
        }
    }
}

impl AudioConverterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for AudioConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Files");
        });
    }
}
