use ffmpeg_next::format;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub enum AlbumArtError {
    NotFound,
    DecodeFailed,
}

#[derive(Clone)]
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
    // TODO: gather files from directories

    pub fn new(path: PathBuf) -> Self {
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
        };
    }

    pub fn ff_get_album_art(&self) -> Result<Option<Vec<u8>>, ffmpeg_next::Error> {
        let mut ff_ctx = format::input(&self.path)?;

        let stream = match ff_ctx.streams().find(|s| {
            s.parameters().medium() == ffmpeg_next::media::Type::Video
                && s.disposition()
                    .contains(ffmpeg_next::format::stream::Disposition::ATTACHED_PIC)
        }) {
            Some(s) => s,
            None => return Ok(None),
        };

        let stream_index = stream.index();

        for (s, packet) in ff_ctx.packets() {
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
            // println!("started getting image");
            let audio_file = AudioFile {
                path,
                ..Default::default()
            };
            let result = audio_file
                .ff_get_album_art()
                .ok()
                .flatten()
                .ok_or(AlbumArtError::NotFound);

            // println!("decoding image...");
            let image = decode_image(&result.unwrap())
                .ok()
                .ok_or(AlbumArtError::DecodeFailed); // TODO: error handle

            // println!("finished getting image, sending via tx");
            let _ = tx.send(image);
        });

        rx
    }
}

// TODO: this is the culprit, image decode is hella slow
fn decode_image(bytes: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    use std::io::Cursor;
    use image::ImageReader;

    let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let img = reader.decode()?.thumbnail_exact(300, 300).to_rgba8();

    Ok(egui::ColorImage::from_rgba_unmultiplied([300, 300], &img))
}

pub fn get_image_hash(bytes: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}
