use ffmpeg_next::format;
use std::path::PathBuf;

use egui::DroppedFile;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
// TODO: consider adding some cover art viewer and/or other metadata fields
pub struct AudioFile {
    pub path: PathBuf,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub filename: String,
}

impl AudioFile {
    pub fn new(path: PathBuf, name: String) -> Self {
        return Self {
            path: path,
            artist: None,
            title: None,
            filename: name,
        };
    }

    // TODO: gather files from directories
    // TODO: add artist - title metadata detection
    pub fn new_from_dropped_file(file: DroppedFile) -> Self {
        return Self {
            path: file.path.unwrap_or_default(),
            artist: None,
            title: None,
            filename: file.name,
        };
    }

    pub fn new_from_pathbuf(path: PathBuf) -> Self {
        // TODO: gather files from directories
        // use std::fs;
        // let metadata = fs::metadata(path.to_string_lossy().to_string());
        // println!("{:#?}", metadata);

        let filename = path
            .file_name()
            .unwrap() // TODO: consider actual error handling lol
            .to_string_lossy()
            .to_string();

        let ff_ctx = format::input(&path).expect("Invalid path provided to FFmpeg");
        let metadata = ff_ctx.metadata();
        let artist: Option<String> = metadata.get("ARTIST").map(|s| s.to_string()); // HACK: bruh
        let title: Option<String> = metadata.get("TITLE").map(|s| s.to_string()); // HACK: bruh

        return Self {
            path: path,
            artist: artist,
            title: title,
            filename: filename,
        };
    }
}
