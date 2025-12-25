use std::sync::mpsc;
use std::thread;

use crate::app;
use crate::models::audio_file::AudioFile;
use crate::transcode;

#[derive(Debug, Clone)]
enum TaskStatus {
    Started,
    Paused,
    Failed,
    Completed,
}

#[derive(Debug)]
pub struct Task {
    file: AudioFile,
    status: Option<mpsc::Receiver<TaskStatus>>,
}

impl Task {
    pub fn new(file: AudioFile) -> Self {
        return Task {
            file,
            status: None,
        };
    }

    pub fn start_transcode(&mut self, settings: &app::Settings) {
        let file = self.file.clone();
        let settings = settings.clone();
        let (tx, rx) = mpsc::channel();

        self.status = Some(rx);
        let _ = tx.send(TaskStatus::Started);

        thread::spawn(move || {
            match transcode::convert_file(
                file,
                &settings.out_codec,
                settings.out_bitrate,
                &settings.out_directory,
                &settings.out_container,
            ) {
                Ok(_) => {
                    let _ = tx.send(TaskStatus::Completed);
                },
                Err(_) => {
                    let _ = tx.send(TaskStatus::Failed);
                },
            }
        });
    }
}
