use egui::TextureHandle;
use ffmpeg_next::format;
use image::imageops::FilterType;
use std::path::PathBuf;

#[derive(Clone)]
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

    // TODO: URGENT, need to optimise or chuck this to a different thread from the UI (main) thread
    pub fn get_album_art(&self) -> Result<Option<Vec<u8>>, ffmpeg_next::Error> {
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

    pub fn load_album_art(&self, ctx: &egui::Context) -> Option<TextureHandle> {
        let placeholder_key = egui::Id::new(("album_art_placeholder", self.path.to_string_lossy().to_string()));

        if let Some(texture) = ctx.data_mut(|data| data.get_temp::<TextureHandle>(placeholder_key)) {
            return Some(texture.clone());
        }

        let bytes = self.get_album_art().ok()??;
        let hash = get_image_hash(&bytes);
        let key = egui::Id::new(("album_art", hash));

        if let Some(texture) = ctx.data(|data| data.get_temp::<TextureHandle>(key)) {
            return Some(texture.clone());
        }

        let image = decode_image(&bytes).ok()?;

        let texture = ctx.load_texture(format!("album_{}", hash), image, egui::TextureOptions::LINEAR);

        ctx.data_mut(|data| {
            data.insert_temp(key, texture.clone());
        });

        Some(texture)
    }
}

fn decode_image(bytes: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory(bytes)?.to_rgba8();
    let resized = image::imageops::resize(
        &image,
        300,
        300,
        FilterType::Lanczos3,
    );

    let size = [resized.width() as usize, resized.height() as usize];

    Ok(egui::ColorImage::from_rgba_unmultiplied(size, &resized))
}

fn get_image_hash(bytes: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}
