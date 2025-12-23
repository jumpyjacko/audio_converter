use ffmpeg_next::{codec, format, media};

use crate::{
    app::AudioConverterApp,
    models::audio_file::{self, AudioFile},
};

// Transcoding code almost word-for-word copied from ffmpeg-next/examples/transcode-audio.rs
struct Transcoder {
    stream: usize,
    decoder: codec::decoder::Audio,
    encoder: codec::encoder::Audio,
}

fn transcoder(
    ictx: &mut format::context::Input,
    octx: &mut format::context::Output,
    settings: &AudioConverterApp,
) -> Result<Transcoder, ffmpeg_next::Error> {
    let input = ictx
        .streams()
        .best(media::Type::Audio)
        .ok_or(ffmpeg_next::Error::StreamNotFound)?;
    let context = codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context.decoder().audio()?;
    let codec = codec::encoder::find(match settings.out_codec {
        audio_file::AudioCodec::FLAC => codec::Id::FLAC,
        audio_file::AudioCodec::MP3 => codec::Id::MP3,
        audio_file::AudioCodec::AAC => codec::Id::AAC,
        audio_file::AudioCodec::OPUS => codec::Id::OPUS,
        audio_file::AudioCodec::VORBIS => codec::Id::VORBIS,
    })
    .ok_or(ffmpeg_next::Error::EncoderNotFound)?
    .audio()?;
    decoder.set_parameters(input.parameters())?;

    let mut output = octx.add_stream(codec)?;
    let context = codec::context::Context::from_parameters(output.parameters())?;
    let mut encoder = context.encoder().audio()?;

    let channel_layout = codec
        .channel_layouts()
        .map(|cls| cls.best(decoder.channel_layout().channels()))
        .unwrap_or(ffmpeg_next::channel_layout::ChannelLayout::STEREO);

    encoder.set_rate(decoder.rate() as i32);
    encoder.set_channel_layout(channel_layout);
    encoder.set_format(
        codec
            .formats()
            .expect("unknown supported formats")
            .next()
            .unwrap(),
    );
    encoder.set_bit_rate(settings.out_bitrate);
    encoder.set_max_bit_rate(decoder.max_bit_rate());

    encoder.set_time_base((1, decoder.rate() as i32));
    output.set_time_base((1, decoder.rate() as i32));

    let encoder = encoder.open_as(codec)?;
    output.set_parameters(&encoder);

    Ok(Transcoder {
        stream: input.index(),
        decoder,
        encoder,
    })
}

pub fn convert_file(
    file: &AudioFile,
    settings: &AudioConverterApp,
) -> Result<(), ffmpeg_next::Error> {
    let mut output_path: String = settings.out_directory.clone() + "/";
    if let Some(stem) = file.path.file_stem().unwrap().to_str() {
        output_path += stem;
    } else {
        output_path += file.title.as_ref().unwrap();
    }

    // input file
    let mut ictx: format::context::Input = format::input(&file.path)?;
    let mut octx = format::output(&output_path)?;
    let transcoder = transcoder(&mut ictx, &mut octx, settings).unwrap();

    octx.set_metadata(ictx.metadata().to_owned());
    octx.write_header().unwrap();

    // TODO: transcode audio stream, copy mjpeg (or png???) / video stream to output
    for (stream, mut packet) in ictx.packets() {}

    Ok(())
}
