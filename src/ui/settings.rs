use crate::models::settings::{AppTheme, OutputGrouping, Settings};
use crate::models::audio_file::{AudioCodec, AudioContainer, AudioSampleRate};
use crate::app::AppState;

pub fn settings_list(settings: &mut Settings, state: &AppState, ui: &mut egui::Ui) {
    egui::Grid::new("settings")
        .num_columns(2)
        // .striped(true)
        .show(ui, |ui| {
            ui.label("Theme");
            egui::ComboBox::from_id_salt("app_theme")
                .selected_text(match settings.app_theme {
                    AppTheme::System => "Follow system",
                    AppTheme::Dark => "Dark",
                    AppTheme::Light => "Light",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.app_theme, AppTheme::System, "Follow system");
                    ui.selectable_value(&mut settings.app_theme, AppTheme::Dark, "Dark");
                    ui.selectable_value(&mut settings.app_theme, AppTheme::Light, "Light");
                });
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            ui.heading("Runtime Settings");
            ui.end_row();

            ui.label("Concurrent tasks");
            ui.add(
                egui::DragValue::new(&mut settings.run_concurrent_task_count)
                    .fixed_decimals(0)
                    .speed(1.0)
                    .range(1..=10),
            );
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            ui.heading("Output settings");
            ui.end_row();

            ui.label("Audio codec");
            egui::ComboBox::from_id_salt("output_codec_combobox")
                .selected_text(match settings.out_codec {
                    AudioCodec::FLAC => "FLAC",
                    AudioCodec::MP3 => "MP3",
                    AudioCodec::AAC => "AAC",
                    AudioCodec::OPUS => "OPUS",
                    AudioCodec::VORBIS => "VORBIS",
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut settings.out_codec,
                            AudioCodec::FLAC,
                            "FLAC",
                        )
                        .clicked()
                    {
                        settings.out_container = AudioContainer::FLAC;
                    }
                    if ui
                        .selectable_value(&mut settings.out_codec, AudioCodec::MP3, "MP3")
                        .clicked()
                    {
                        settings.out_container = AudioContainer::MP3;
                        if settings.out_sample_rate == AudioSampleRate::HiRes96 {
                            settings.out_sample_rate = AudioSampleRate::Studio48;
                        }
                    }
                    if ui
                        .selectable_value(&mut settings.out_codec, AudioCodec::AAC, "AAC")
                        .clicked()
                    {
                        settings.out_container = AudioContainer::M4A;
                        if settings.out_sample_rate == AudioSampleRate::HiRes96 {
                            settings.out_sample_rate = AudioSampleRate::Studio48;
                        }
                    }
                    if ui
                        .selectable_value(
                            &mut settings.out_codec,
                            AudioCodec::OPUS,
                            "OPUS",
                        )
                        .clicked()
                    {
                        settings.out_container = AudioContainer::OGG;
                        if settings.out_sample_rate == AudioSampleRate::HiRes96 {
                            settings.out_sample_rate = AudioSampleRate::Studio48;
                        }
                    }
                    if ui
                        .selectable_value(
                            &mut settings.out_codec,
                            AudioCodec::VORBIS,
                            "VORBIS",
                        )
                        .clicked()
                    {
                        settings.out_container = AudioContainer::OGG;
                    };
                });
            ui.end_row();

            ui.label("Audio container");
            egui::ComboBox::from_id_salt("output_container_combobox")
                .selected_text(match settings.out_container {
                    AudioContainer::FLAC => ".flac",
                    AudioContainer::MP3 => ".mp3",
                    AudioContainer::M4A => ".m4a",
                    AudioContainer::OGG => ".ogg",
                    AudioContainer::OPUS => ".opus",
                })
                .show_ui(ui, |ui| match settings.out_codec {
                    AudioCodec::FLAC => {
                        ui.selectable_value(
                            &mut settings.out_container,
                            AudioContainer::FLAC,
                            ".flac",
                        );
                    }
                    AudioCodec::MP3 => {
                        ui.selectable_value(
                            &mut settings.out_container,
                            AudioContainer::MP3,
                            ".mp3",
                        );
                    }
                    AudioCodec::AAC => {
                        ui.selectable_value(
                            &mut settings.out_container,
                            AudioContainer::M4A,
                            ".m4a",
                        );
                    }
                    AudioCodec::OPUS => {
                        ui.selectable_value(
                            &mut settings.out_container,
                            AudioContainer::OPUS,
                            ".opus",
                        );
                        ui.selectable_value(
                            &mut settings.out_container,
                            AudioContainer::OGG,
                            ".ogg",
                        );
                    }
                    AudioCodec::VORBIS => {
                        ui.selectable_value(
                            &mut settings.out_container,
                            AudioContainer::OGG,
                            ".ogg",
                        );
                    }
                });
            ui.end_row();

            ui.label("Sample rate");
            egui::ComboBox::from_id_salt("output_samplerate_combobox")
                .selected_text(match settings.out_sample_rate {
                    AudioSampleRate::CD44 => "CD (44.1kHz)",
                    AudioSampleRate::Studio48 => "Studio (48kHz)",
                    AudioSampleRate::HiRes96 => "HiRes (96kHz)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.out_sample_rate, AudioSampleRate::CD44, "CD (44.1kHz)");
                    ui.selectable_value(&mut settings.out_sample_rate, AudioSampleRate::Studio48, "Studio (48kHz)");

                    if settings.out_codec == AudioCodec::FLAC {
                        ui.selectable_value(&mut settings.out_sample_rate, AudioSampleRate::HiRes96, "HiRes (96kHz)");
                    }
                });
            ui.end_row();

            ui.label("Bitrate");
            ui.add(
                egui::DragValue::new(&mut settings.out_bitrate)
                    .fixed_decimals(0)
                    .speed(1000.0),
            );
            ui.end_row();

            let text_width = ui.available_width().min(240.0);

            ui.label("Output Directory");
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [text_width, ui.text_style_height(&egui::TextStyle::Body)],
                        egui::TextEdit::singleline(&mut settings.out_directory),
                    )
                    .double_clicked()
                    || ui.button("🗁").clicked()
                {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        settings.out_directory = dir.to_str().unwrap().to_string();
                    }
                }
            });
            ui.end_row();

            ui.label("Group by...")
                .on_hover_text_at_pointer("Group output files in a folder");
            egui::ComboBox::from_id_salt("output_grouping_combobox")
                .selected_text(match settings.out_grouping {
                    OutputGrouping::NoGrouping => "No Grouping",
                    OutputGrouping::Copy => "Copy from source",
                    OutputGrouping::ArtistAlbum => "Artist - Album",
                    OutputGrouping::Album => "Album",
                    OutputGrouping::Artist => "Artist",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut settings.out_grouping,
                        OutputGrouping::NoGrouping,
                        "No Grouping",
                    )
                    .on_hover_text_at_pointer(
                        "Group output files in a folder:\n - No grouping/folders",
                    );
                    ui.selectable_value(
                        &mut settings.out_grouping,
                        OutputGrouping::Copy,
                        "Copy from source",
                    )
                    .on_hover_text_at_pointer(
                        "Group output files in a folder:\n - Parent folder from original files",
                    );
                    ui.selectable_value(
                        &mut settings.out_grouping,
                        OutputGrouping::ArtistAlbum,
                        "Artist - Album",
                    )
                    .on_hover_text_at_pointer({
                        let first_file = state.files.first();
                        let artist = first_file.and_then(|f| f.artist.as_deref()).unwrap_or("Artist");
                        let album = first_file.and_then(|f| f.album.as_deref()).unwrap_or("Album");
                        format!("Group output files in a folder:\n - Create a folder name '{artist} - {album}'")
                    });
                    ui.selectable_value(
                        &mut settings.out_grouping,
                        OutputGrouping::Album,
                        "Album",
                    )
                    .on_hover_text_at_pointer({
                        let first_file = state.files.first();
                        let album = first_file.and_then(|f| f.album.as_deref()).unwrap_or("Album");
                        format!("Group output files in a folder:\n - Create a folder name '{album}'")
                    });
                    ui.selectable_value(
                        &mut settings.out_grouping,
                        OutputGrouping::Artist,
                        "Artist",
                    ).on_hover_text_at_pointer({
                        let first_file = state.files.first();
                        let artist = first_file.and_then(|f| f.artist.as_deref()).unwrap_or("Artist");
                        format!("Group output files in a folder:\n - Create a folder name '{artist}'")
                    });
                }).response.on_hover_text_at_pointer("Group output files in a folder");
            ui.end_row();

            let cover_art_tooltip = "Toggle embedding cover art as a Vorbis metadata block\n - depending on the source file, it may inflate file size";
            ui.label("Embed cover art").on_hover_text_at_pointer(cover_art_tooltip);
            ui.checkbox(&mut settings.out_embed_art, "").on_hover_text_at_pointer(cover_art_tooltip);
            ui.end_row();

            let resize_cover_art_tooltip = "Resize the embedded cover art and compress to Jpeg, reduces final file size";
            ui.add_enabled_ui(settings.out_embed_art, |ui| {
                ui.label("Resize cover art?").on_hover_text_at_pointer(resize_cover_art_tooltip);
            });
            ui.add_enabled_ui(settings.out_embed_art, |ui| {
                ui.checkbox(&mut settings.out_enable_cover_art_resize, "").on_hover_text_at_pointer(resize_cover_art_tooltip);
            });
            ui.end_row();

            ui.add_enabled_ui(settings.out_enable_cover_art_resize && settings.out_embed_art, |ui| {
                ui.label("Cover art resolution");
            });
            ui.add_enabled_ui(settings.out_enable_cover_art_resize && settings.out_embed_art, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut settings.out_cover_art_resolution)
                            .fixed_decimals(0)
                            .speed(10.0)
                    );
                    ui.label("px");
                });
            });
            ui.end_row();
        });
}
