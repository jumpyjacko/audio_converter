use std::sync::mpsc;
use std::thread;

use crate::app;
use crate::models::audio_file::AudioFile;
use crate::transcode;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TaskStatus {
    Started,
    Paused,
    Failed,
    Completed,
}

#[derive(Debug)]
pub struct Task {
    pub file: AudioFile,
    status: Option<mpsc::Receiver<TaskStatus>>,
}

impl Task {
    pub fn new(file: AudioFile) -> Self {
        return Task { file, status: None };
    }

    pub fn start_transcode(&mut self, settings: &app::Settings) {
        let file = self.file.clone();
        let settings = settings.clone();
        let (tx, rx) = mpsc::channel();

        let _ = tx.send(TaskStatus::Started);
        self.status = Some(rx);

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
                }
                Err(_) => {
                    let _ = tx.send(TaskStatus::Failed);
                }
            }
        });
    }

    pub fn is_complete(&self) -> bool {
        let Some(rx) = &self.status else {
            return false;
        };

        match rx.try_recv() {
            Ok(TaskStatus::Completed) | Ok(TaskStatus::Failed) => true,
            Ok(TaskStatus::Paused) => false,
            Ok(TaskStatus::Started) => false,
            Err(mpsc::TryRecvError::Empty) => false,
            Err(mpsc::TryRecvError::Disconnected) => true, // worker died
        }
    }
}
