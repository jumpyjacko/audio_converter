use crate::tasks_manager::TasksManager;

pub fn task_queue_window(tasks_manager: &mut TasksManager, ctx: &egui::Context) {
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
                tasks_manager.queue.len() + tasks_manager.active_tasks.len()
            ));
            ui.separator();

            for task in &tasks_manager.active_tasks {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{} - {} on {}",
                        task.file.artist.clone().unwrap_or(crate::app::NO_ARTIST.to_string()),
                        task.file.title.clone().unwrap_or(crate::app::NO_TITLE.to_string()),
                        task.file.album.clone().unwrap_or(crate::app::NO_ALBUM.to_string())
                    ));
                });
            }
        });
}
