use std::path::PathBuf;

use egui::DroppedFile;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
// TODO: consider adding some cover art viewer and/or other metadata fields
pub struct AudioFile {
    pub path: PathBuf,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub filename: String,
    pub mime: String,
}

impl AudioFile {
    pub fn new(path: PathBuf, name: String, mime: String) -> Self {
        return Self {
            path: path,
            artist: None,
            title: None,
            filename: name,
            mime,
        };
    }

    // TODO: add artist - title metadata detection
    pub fn new_from_dropped_file(file: DroppedFile) -> Self {
        return Self {
            path: file.path.unwrap_or_default(),
            artist: None,
            title: None,
            filename: file.name,
            mime: file.mime,
        };
    }

    pub fn new_from_pathbuf(path: PathBuf) -> Self {
        return Self {
            path: path,
            artist: None,
            title: None,
            filename: "To be done".to_string(),
            mime: "To be done".to_string(),
        }
    }
}
