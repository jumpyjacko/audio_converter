use ffmpeg_next::{format, media};
use image::{ImageBuffer, Rgba};
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
    NoExtension,
    InputError,
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

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq, Clone)]
pub enum AudioSampleRate {
    CD44,
    Studio48,
    HiRes96,
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
        let Some(path_ext) = &path.extension() else {
            return Err(AudioFileError::NoExtension);
        };
        let Some(path_ext) = &path_ext.to_str() else {
            return Err(AudioFileError::InputError);
        };
        if !ALLOWED_INPUT_TYPES.contains(path_ext) {
            return Err(AudioFileError::NotAnAudioFile);
        }

        let input_ctx = format::input(&path).expect("Invalid path provided to FFmpeg"); // TODO: loading metadata makes up all the loading time, its instant without it

        return Ok(Self {
            path: path,
            artist: get_tag(&input_ctx, "ARTIST"),
            album: get_tag(&input_ctx, "ALBUM"),
            title: get_tag(&input_ctx, "TITLE"),
            track: get_tag(&input_ctx, "TRACK"),
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
                    Err(_) => panic!("hdwgh?"), // FIX: real error, happens on hidden folders?
                };
                files.push(audio_file);
            }
        }

        files.sort_unstable_by_key(|f| f.track.as_deref().and_then(parse_track_string)); // TODO: this adds like 4 seconds on a 1.7k file load

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

    pub fn load_album_art(
        &self,
        size: Option<u32>,
    ) -> mpsc::Receiver<Result<egui::ColorImage, AlbumArtError>> {
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

            let decoded = match decode_thumbnail(&result.unwrap(), size) {
                Ok(image) => image,
                Err(_) => return Err(AlbumArtError::DecodeFailed),
            };
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [decoded.width() as usize, decoded.height() as usize],
                &decoded,
            );

            let _ = tx.send(Ok(image));

            Ok(())
        });

        rx
    }
}

fn get_tag(ctx: &format::context::Input, key: &str) -> Option<String> {
    if let Some(v) = ctx.metadata().get(key) {
        return Some(v.to_string());
    }

    let stream = ctx.streams().best(media::Type::Audio)?;
    if let Some(v) = stream.metadata().get(key) {
        return Some(v.to_string());
    }

    None
}

pub fn decode_thumbnail(
    bytes: &[u8],
    size: Option<u32>,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, image::ImageError> {
    use image::ImageReader;
    use std::io::Cursor;

    let img = {
        let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
        let decoded = reader.decode()?;

        match size {
            Some(s) => decoded.thumbnail_exact(s, s).to_rgba8(),
            None => decoded.to_rgba8(),
        }
    };

    Ok(img)
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
