use ffmpeg_next::format;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

pub const ALLOWED_INPUT_TYPES: [&str; 7] = ["flac", "mp3", "ogg", "wav", "opus", "aac", "m4a"];

#[derive(Debug)]
pub enum AlbumArtError {
    NotFound,
    DecodeFailed,
}

#[derive(Debug)]
pub enum AudioFileError {
    NotAnAudioFile,
    NotADirectory,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq, Clone)]
pub enum AudioCodec {
    FLAC,
    MP3,
    AAC,
    OPUS,
    VORBIS,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq, Clone)]
pub enum AudioContainer {
    FLAC,
    MP3,
    M4A,
    OPUS,
    OGG,
}

#[derive(Clone, Debug)]
pub struct AudioFile {
    pub path: PathBuf,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub track: Option<String>,
}

impl Default for AudioFile {
    fn default() -> Self {
        Self {
            path: Default::default(),
            artist: Default::default(),
            album: Default::default(),
            title: Default::default(),
            track: Default::default(),
        }
    }
}

impl AudioFile {
    pub fn new(path: PathBuf) -> Result<Self, AudioFileError> {
        if !ALLOWED_INPUT_TYPES.contains(&path.extension().unwrap().to_str().unwrap()) {
            // TODO: literally what am i writing
            return Err(AudioFileError::NotAnAudioFile);
        }

        let input_ctx = format::input(&path).expect("Invalid path provided to FFmpeg");
        let metadata = input_ctx.metadata();
        let artist: Option<String> = metadata.get("ARTIST").map(|s| s.to_string());
        let album: Option<String> = metadata.get("ALBUM").map(|s| s.to_string());
        let title: Option<String> = metadata.get("TITLE").map(|s| s.to_string());
        let track: Option<String> = metadata.get("track").map(|s| s.to_string());

        return Ok(Self {
            path: path,
            artist: artist,
            album: album,
            title: title,
            track: track,
        });
    }

    pub fn from_directory(path: &PathBuf) -> Result<Vec<Self>, AudioFileError> {
        if !path.is_dir() {
            return Err(AudioFileError::NotADirectory);
        }

        let mut files: Vec<Self> = Vec::new();

        for file in path.read_dir().unwrap() {
            if let Ok(file) = file {
                let audio_file = match AudioFile::new(file.path()) {
                    Ok(af) => af,
                    Err(AudioFileError::NotAnAudioFile) => continue,
                    Err(_) => panic!("hdwgh?"),
                };
                files.push(audio_file);
            }
        }

        files.sort_unstable_by_key(|f| f.track.as_deref().and_then(parse_track_string));

        return Ok(files);
    }

    pub fn ff_get_album_art(&self) -> Result<Option<Vec<u8>>, ffmpeg_next::Error> {
        let mut input_ctx = format::input(&self.path)?;

        let stream = match input_ctx.streams().find(|s| {
            s.parameters().medium() == ffmpeg_next::media::Type::Video
                && s.disposition()
                    .contains(ffmpeg_next::format::stream::Disposition::ATTACHED_PIC)
        }) {
            Some(s) => s,
            None => return Ok(None),
        };

        let stream_index = stream.index();

        for (s, packet) in input_ctx.packets() {
            if s.index() == stream_index {
                return Ok(Some(packet.data().unwrap().to_vec()));
            }
        }

        Ok(None)
    }

    pub fn load_album_art(&self) -> mpsc::Receiver<Result<egui::ColorImage, AlbumArtError>> {
        let (tx, rx) = mpsc::channel();
        let path = self.path.clone();

        thread::spawn(move || {
            let audio_file = AudioFile {
                path,
                ..Default::default()
            };
            let result = audio_file
                .ff_get_album_art()
                .ok()
                .flatten()
                .ok_or(AlbumArtError::NotFound);

            let image = to_egui_colorimage(&result.unwrap())
                .ok()
                .ok_or(AlbumArtError::DecodeFailed); // TODO: error handle

            let _ = tx.send(image);
        });

        rx
    }
}

pub fn to_egui_colorimage(bytes: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    use image::ImageReader;
    use std::io::Cursor;

    let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let img = reader.decode()?.thumbnail_exact(300, 300).to_rgba8();

    Ok(egui::ColorImage::from_rgba_unmultiplied([300, 300], &img))
}

// pub fn get_image_hash(bytes: &[u8]) -> u64 {
//     use std::collections::hash_map::DefaultHasher;
//     use std::hash::{Hash, Hasher};
//
//     let mut hasher = DefaultHasher::new();
//     bytes.hash(&mut hasher);
//     hasher.finish()
// }

// TODO: handle strange track number strings?? i've never encountered other types than '1' and '1/12'
fn parse_track_string(s: &str) -> Option<u32> {
    s.split('/').next().and_then(|n| n.parse::<u32>().ok())
}
