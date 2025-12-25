use std::thread;

use crate::app;
use crate::models::audio_file::AudioFile;
use crate::transcode;

#[derive(Debug, Clone)]
enum TaskStatus {
    NotStarted,
    Started,
    Paused,
    Failed,
    Completed,
}

#[derive(Debug, Clone)]
pub struct Task {
    file: AudioFile,
    progress: u8,
    status: TaskStatus,
}

impl Task {
    pub fn new(file: AudioFile) -> Self {
        return Task {
            file,
            progress: 0,
            status: TaskStatus::NotStarted,
        };
    }

    pub fn start_transcode(&mut self, settings: &app::Settings) {
        let file = self.file.clone();
        let settings = settings.clone();

        thread::spawn(move || {
            let _ = transcode::convert_file(
                file,
                &settings.out_codec,
                settings.out_bitrate,
                &settings.out_directory,
                &settings.out_container,
            );
        });
    }
}
