use ffmpeg_next::format;
use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct AudioFile {
    pub path: PathBuf,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub track: Option<String>,
    pub filename: String,
}

impl AudioFile {
    // TODO: gather files from directories

    pub fn new(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .unwrap() // TODO: consider actual error handling lol
            .to_string_lossy()
            .to_string();

        let ff_ctx = format::input(&path).expect("Invalid path provided to FFmpeg");
        let metadata = ff_ctx.metadata();
        let artist: Option<String> = metadata.get("ARTIST").map(|s| s.to_string()); // HACK: bruh
        let album: Option<String> = metadata.get("ALBUM").map(|s| s.to_string()); // HACK: bruh
        let title: Option<String> = metadata.get("TITLE").map(|s| s.to_string()); // HACK: bruh
        let track: Option<String> = metadata.get("track").map(|s| s.to_string()); // HACK: bruh

        return Self {
            path: path,
            artist: artist,
            album: album,
            title: title,
            track: track,
            filename: filename,
        };
    }
}
