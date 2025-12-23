use std::collections::VecDeque;

use crate::models::audio_file::AudioFile;

enum TaskStatus {
    NotStarted,
    Started,
    Paused,
    Failed,
    Completed
}

pub struct Task<'a> {
    file: &'a AudioFile,
    progress: u8,
    status: TaskStatus
}

pub struct TaskQueue<'a> {
    queue: VecDeque<Task<'a>>,
}

impl TaskQueue<'_> {
    pub fn new() -> Self {
        return TaskQueue {
            queue: VecDeque::new(),
        }
    }
}
