use std::sync::mpsc;

use egui::{Sense, Vec2};

use crate::app::AppState;

pub fn file_info_popup(state: &mut AppState, ctx: &egui::Context) {
    use egui::Align2;

    let file = state
        .files
        .get(state.last_selection.unwrap())
        .unwrap()
        .clone();

    egui::Window::new("File information")
        .min_width(300.0)
        .max_width(300.0)
        .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
        .resizable(false)
        .movable(false)
        .default_open(false)
        .show(ctx, |ui| {
            ui.heading(
                file.title
                    .clone()
                    .unwrap_or(crate::app::NO_TITLE.to_string()),
            );
            egui::Grid::new("detailed_file_info")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Artist:");
                    ui.label(
                        file.artist
                            .clone()
                            .unwrap_or(crate::app::NO_ARTIST.to_string()),
                    );
                    ui.end_row();

                    ui.label("Album:");
                    ui.label(
                        file.album
                            .clone()
                            .unwrap_or(crate::app::NO_ALBUM.to_string()),
                    );
                    ui.end_row();

                    ui.label("File path:");
                    ui.add(
                        egui::Label::new(file.path.clone().to_string_lossy().to_string()).wrap(),
                    );
                    ui.end_row();
                });

            ui.separator();

            if let Some(rx) = &state.cover_art_rx {
                match rx.try_recv() {
                    Ok(Ok(image)) => {
                        let texture =
                            ctx.load_texture("cover_art", image, egui::TextureOptions::LINEAR);

                        state.cover_art = Some(texture);
                        state.cover_art_rx = None;

                        ctx.request_repaint();
                    }
                    Ok(Err(_)) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        state.cover_art = None;
                        state.cover_art_rx = None;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        let _ = ui.label("Loading image...");
                        state.cover_art = None;
                    }
                }
            }

            if let Some(texture) = &state.cover_art {
                let response = ui.add(
                    egui::Image::from_texture(texture)
                        .fit_to_fraction(Vec2::ONE)
                        .max_width(300.0)
                        .corner_radius(5)
                        .sense(Sense::CLICK),
                );

                if response.clicked() {
                    state.lg_cover_art_rx = Some(file.load_album_art(None));
                    state.showing_lg_art = true;
                }
            }
        });
}
