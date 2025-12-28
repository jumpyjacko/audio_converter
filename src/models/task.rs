use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use crate::app::{self, NO_ALBUM, NO_ARTIST};
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

        let out_dir: PathBuf = match settings.out_grouping {
            app::OutputGrouping::NoGrouping => PathBuf::from(settings.out_directory),
            app::OutputGrouping::Copy => {
                let mut out = PathBuf::from(&settings.out_directory);
                if let Some(parent) = self.file.path.parent() {
                    if let Some(direct_parent) = parent.file_name().and_then(|f| f.to_str()) {
                        out.push(&direct_parent);
                    };
                }

                out
            }
            app::OutputGrouping::ArtistAlbum => {
                let mut out = PathBuf::from(settings.out_directory);
                let directory = format!(
                    "{} - {}",
                    self.file.artist.as_deref().unwrap_or(NO_ARTIST),
                    self.file.album.as_deref().unwrap_or(NO_ALBUM)
                );
                out.push(directory);
                out
            }
            app::OutputGrouping::Album => {
                let mut out = PathBuf::from(settings.out_directory);
                out.push(self.file.album.as_deref().unwrap_or(NO_ALBUM));
                out
            }
            app::OutputGrouping::Artist => {
                let mut out = PathBuf::from(settings.out_directory);
                out.push(self.file.artist.as_deref().unwrap_or(NO_ARTIST));
                out
            }
        };

        use std::fs;
        let _ = fs::create_dir(&out_dir);

        thread::spawn(move || {
            match transcode::convert_file(
                file,
                &settings.out_codec,
                settings.out_bitrate,
                &out_dir,
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
            Err(mpsc::TryRecvError::Disconnected) => true,
        }
    }
}
