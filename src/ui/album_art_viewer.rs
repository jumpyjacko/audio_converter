use std::sync::mpsc;

use egui::pos2;

use crate::app::AppState;

pub fn large_album_art_viewer(state: &mut AppState, ctx: &egui::Context) {
    use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.showing_lg_art = false;
        state.lg_cover_art_rx = None;
        state.lg_cover_art = None;
    }

    let painter = ctx.layer_painter(LayerId::new(
        Order::Foreground,
        Id::new("large_album_art_viewer"),
    ));

    let content_rect = ctx.content_rect();
    painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));

    if let Some(rx) = &state.lg_cover_art_rx {
        match rx.try_recv() {
            Ok(Ok(image)) => {
                let texture = ctx.load_texture("lg_cover_art", image, egui::TextureOptions::LINEAR);

                state.lg_cover_art = Some(texture);
                state.lg_cover_art_rx = None;

                ctx.request_repaint();
            }
            Ok(Err(_)) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                state.lg_cover_art = None;
                state.lg_cover_art_rx = None;
            }
            Err(mpsc::TryRecvError::Empty) => {
                let _ = painter.text(
                    content_rect.center(),
                    Align2::CENTER_CENTER,
                    "Loading image...",
                    TextStyle::Heading.resolve(&ctx.style()),
                    Color32::WHITE,
                );
                state.lg_cover_art = None;
            }
        }
    }

    if let Some(texture) = &state.lg_cover_art {
        let margin = 32.0;

        let available = content_rect.shrink(margin);
        let tex_size = texture.size_vec2();

        let scale = (available.width() / tex_size.x).min(available.height() / tex_size.y);

        let size = tex_size * scale;
        let dest_rect = egui::Rect::from_center_size(content_rect.center(), size);

        painter.image(
            texture.id(),
            dest_rect,
            egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        let clicked_on_art = ctx.input(|i| i.pointer.any_pressed())
            && !dest_rect.contains(ctx.input(|i| i.pointer.interact_pos()).unwrap());
        if clicked_on_art {
            state.showing_lg_art = false;
            state.lg_cover_art_rx = None;
            state.lg_cover_art = None;
        }
    }
}
