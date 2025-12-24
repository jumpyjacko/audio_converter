use std::collections::VecDeque;

use crate::models::{audio_file::AudioFile, task::Task};

#[derive(Debug)]
pub struct TasksManager {
    queue: VecDeque<Task>,
}

impl TasksManager {
    pub fn new() -> Self {
        return TasksManager { queue: VecDeque::new() }
    }

    pub fn queue_audio_file(&mut self, file: AudioFile) {
        let task = Task::new(file);
        self.queue.push_back(task);
    }
}
