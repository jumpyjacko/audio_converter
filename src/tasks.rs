use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use crate::app::{NO_ALBUM, NO_ARTIST, Settings};
use crate::audio_file::AudioFile;
use crate::transcode;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub enum OutputGrouping {
    NoGrouping,
    Copy,
    ArtistAlbum,
    Album,
    Artist,
}

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

    pub fn start_transcode(&mut self, settings: &Settings) {
        let file = self.file.clone();
        let settings = settings.clone();
        let (tx, rx) = mpsc::channel();

        let _ = tx.send(TaskStatus::Started);
        self.status = Some(rx);

        let out_dir: PathBuf = match settings.out_grouping {
            OutputGrouping::NoGrouping => PathBuf::from(settings.out_directory),
            OutputGrouping::Copy => {
                let mut out = PathBuf::from(&settings.out_directory);
                if let Some(parent) = self.file.path.parent() {
                    if let Some(direct_parent) = parent.file_name().and_then(|f| f.to_str()) {
                        out.push(&direct_parent);
                    };
                }

                out
            }
            OutputGrouping::ArtistAlbum => {
                let mut out = PathBuf::from(settings.out_directory);
                let directory = format!(
                    "{} - {}",
                    self.file.artist.as_deref().unwrap_or(NO_ARTIST),
                    self.file.album.as_deref().unwrap_or(NO_ALBUM)
                );
                out.push(directory);
                out
            }
            OutputGrouping::Album => {
                let mut out = PathBuf::from(settings.out_directory);
                out.push(self.file.album.as_deref().unwrap_or(NO_ALBUM));
                out
            }
            OutputGrouping::Artist => {
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
                &settings.out_sample_rate,
                settings.out_bitrate,
                &out_dir,
                &settings.out_container,
                settings.out_embed_art,
                settings.out_enable_cover_art_resize,
                settings.out_cover_art_resolution,
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
    pub fn update(&mut self, settings: &Settings) {
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
