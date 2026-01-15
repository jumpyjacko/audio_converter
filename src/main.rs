#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod models;
mod transcode;
mod tasks_manager;
mod ui;

use app::AudioConverterApp;

use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 400.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    ffmpeg_next::init().expect("Failed to initialise FFmpeg");

    eframe::run_native(
        "Batch Audio File Converter",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(AudioConverterApp::new(cc)))
        }),
    )
}
