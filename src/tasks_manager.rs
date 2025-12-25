use std::collections::VecDeque;

use crate::app;
use crate::models::{audio_file::AudioFile, task::Task};

#[derive(Debug)]
pub struct TasksManager {
    pub queue: VecDeque<Task>,
    pub active_tasks: Vec<Task>,
}

impl TasksManager {
    pub fn new() -> Self {
        return TasksManager {
            queue: VecDeque::new(),
            active_tasks: Vec::new(),
        };
    }

    pub fn queue_audio_file(&mut self, file: AudioFile) {
        let task = Task::new(file);
        self.queue.push_back(task);
    }

    /// Updates the active_tasks pool according to settings, called every frame
    pub fn update(&mut self, settings: &app::Settings) {
        self.active_tasks.retain(|task| !task.is_complete());

        while self.active_tasks.len() < settings.run_concurrent_task_count {
            let mut task = match self.queue.pop_front() {
                Some(t) => t,
                None => break,
            };

            task.start_transcode(settings);
            self.active_tasks.push(task);
        }
    }
}
