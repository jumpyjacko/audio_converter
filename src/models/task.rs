use crate::models::audio_file::AudioFile;

#[derive(Debug)]
enum TaskStatus {
    NotStarted,
    Started,
    Paused,
    Failed,
    Completed
}

#[derive(Debug)]
pub struct Task {
    file: AudioFile,
    progress: u8,
    status: TaskStatus
}

impl Task {
    pub fn new(file: AudioFile) -> Self {
        return Task {
            file,
            progress: 0,
            status: TaskStatus::NotStarted,
        }
    }
}
