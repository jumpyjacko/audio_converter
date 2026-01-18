use crate::models::audio_file::{AudioCodec, AudioContainer, AudioSampleRate};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub enum AppTheme {
    System,
    Dark,
    Light,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub enum OutputGrouping {
    NoGrouping,
    Copy,
    ArtistAlbum,
    Album,
    Artist,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Settings {
    pub app_theme: AppTheme,

    pub run_concurrent_task_count: usize,

    pub out_codec: AudioCodec,
    pub out_container: AudioContainer,
    pub out_sample_rate: AudioSampleRate,
    pub out_bitrate: usize,
    pub out_directory: String,
    pub out_grouping: OutputGrouping,
    pub out_embed_art: bool,
    pub out_enable_cover_art_resize: bool,
    pub out_cover_art_resolution: u32,
}
