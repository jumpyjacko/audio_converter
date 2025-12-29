use std::path::Path;

use base64::prelude::*;
use byteorder::{BigEndian, WriteBytesExt};
use ffmpeg_next::{codec, filter, format, frame, media};

use crate::models::audio_file::{self, AudioCodec, AudioContainer, AudioFile};

// Transcoding code almost word-for-word copied from ffmpeg-next/examples/transcode-audio.rs
struct Transcoder {
    stream: usize,
    filter: filter::Graph,
    decoder: codec::decoder::Audio,
    encoder: codec::encoder::Audio,
    in_time_base: ffmpeg_next::Rational,
    out_time_base: ffmpeg_next::Rational,
}

fn filter(
    decoder: &codec::decoder::Audio,
    encoder: &codec::encoder::Audio,
) -> Result<filter::Graph, ffmpeg_next::Error> {
    let mut filter = filter::Graph::new();

    let in_args = format!(
        "time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
        decoder.time_base(),
        decoder.rate(),
        decoder.format().name(),
        decoder.channel_layout().bits()
    );

    filter.add(&filter::find("abuffer").unwrap(), "in", &in_args)?;
    filter.add(&filter::find("abuffersink").unwrap(), "out", "")?;

    let filter_spec = format!(
        "aresample={},aformat=sample_fmts={}:channel_layouts={}",
        encoder.rate(),
        encoder.format().name(),
        encoder.channel_layout().bits(),
    );

    filter
        .output("in", 0)?
        .input("out", 0)?
        .parse(&filter_spec)?;
    filter.validate()?;

    if let Some(codec) = encoder.codec() {
        if !codec
            .capabilities()
            .contains(ffmpeg_next::codec::capabilities::Capabilities::VARIABLE_FRAME_SIZE)
        {
            filter
                .get("out")
                .unwrap()
                .sink()
                .set_frame_size(encoder.frame_size());
        }
    }

    Ok(filter)
}

fn transcoder(
    ictx: &mut format::context::Input,
    octx: &mut format::context::Output,
    out_codec: &AudioCodec,
    out_bitrate: usize,
) -> Result<Transcoder, ffmpeg_next::Error> {
    let input = ictx
        .streams()
        .best(media::Type::Audio)
        .ok_or(ffmpeg_next::Error::StreamNotFound)?;
    let context = codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context.decoder().audio()?;
    let codec = codec::encoder::find(match out_codec {
        audio_file::AudioCodec::FLAC => codec::Id::FLAC,
        audio_file::AudioCodec::MP3 => codec::Id::MP3,
        audio_file::AudioCodec::AAC => codec::Id::AAC,
        audio_file::AudioCodec::OPUS => codec::Id::OPUS,
        audio_file::AudioCodec::VORBIS => codec::Id::VORBIS,
    })
    .ok_or(ffmpeg_next::Error::EncoderNotFound)?
    .audio()?;
    let global = octx
        .format()
        .flags()
        .contains(ffmpeg_next::format::flag::Flags::GLOBAL_HEADER);
    decoder.set_parameters(input.parameters())?;

    let mut output = octx.add_stream(codec)?;
    let context = codec::context::Context::from_parameters(output.parameters())?;
    let mut encoder = context.encoder().audio()?;

    let channel_layout = codec
        .channel_layouts()
        .map(|cls| cls.best(decoder.channel_layout().channels()))
        .unwrap_or(ffmpeg_next::channel_layout::ChannelLayout::STEREO);

    if global {
        encoder.set_flags(ffmpeg_next::codec::flag::Flags::GLOBAL_HEADER);
    }

    if out_codec == &audio_file::AudioCodec::OPUS {
        encoder.set_rate(48000);
    } else {
        encoder.set_rate(decoder.rate() as i32);
    }
    encoder.set_channel_layout(channel_layout);
    encoder.set_format(
        codec
            .formats()
            .expect("unknown supported formats")
            .next()
            .unwrap(),
    );
    encoder.set_bit_rate(out_bitrate);
    encoder.set_max_bit_rate(decoder.max_bit_rate());

    encoder.set_time_base((1, decoder.rate() as i32));
    output.set_time_base((1, decoder.rate() as i32));

    let encoder = encoder.open_as(codec)?;
    output.set_parameters(&encoder);

    let filter = filter(&decoder, &encoder)?;

    let in_time_base = decoder.time_base();
    let out_time_base = output.time_base();

    Ok(Transcoder {
        stream: input.index(),
        filter,
        decoder,
        encoder,
        in_time_base,
        out_time_base,
    })
}

impl Transcoder {
    fn send_frame_to_encoder(&mut self, frame: &ffmpeg_next::Frame) {
        self.encoder.send_frame(frame).unwrap();
    }

    fn send_eof_to_encoder(&mut self) {
        self.encoder.send_eof().unwrap();
    }

    fn receive_and_process_encoded_packets(&mut self, octx: &mut format::context::Output) {
        let mut encoded = ffmpeg_next::Packet::empty();
        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(0);
            encoded.rescale_ts(self.in_time_base, self.out_time_base);
            encoded.write_interleaved(octx).unwrap();
        }
    }

    fn add_frame_to_filter(&mut self, frame: &ffmpeg_next::Frame) {
        self.filter.get("in").unwrap().source().add(frame).unwrap();
    }

    fn flush_filter(&mut self) {
        self.filter.get("in").unwrap().source().flush().unwrap();
    }

    fn get_and_process_filtered_frames(&mut self, octx: &mut format::context::Output) {
        let mut filtered = frame::Audio::empty();
        while self
            .filter
            .get("out")
            .unwrap()
            .sink()
            .frame(&mut filtered)
            .is_ok()
        {
            self.send_frame_to_encoder(&filtered);
            self.receive_and_process_encoded_packets(octx);
        }
    }

    fn send_packet_to_decoder(&mut self, packet: &ffmpeg_next::Packet) {
        self.decoder.send_packet(packet).unwrap();
    }

    fn send_eof_to_decoder(&mut self) {
        self.decoder.send_eof().unwrap();
    }

    fn receive_and_process_decoded_frames(&mut self, octx: &mut format::context::Output) {
        let mut decoded = frame::Audio::empty();
        while self.decoder.receive_frame(&mut decoded).is_ok() {
            let timestamp = decoded.timestamp();
            decoded.set_pts(timestamp);
            self.add_frame_to_filter(&decoded);
            self.get_and_process_filtered_frames(octx);
        }
    }
}

pub fn convert_file(
    file: AudioFile,
    out_codec: &AudioCodec,
    out_bitrate: usize,
    out_directory: &Path,
    out_container: &AudioContainer,
    embed_cover_art: bool,
) -> Result<(), ffmpeg_next::Error> {
    let mut output_path: String = out_directory.to_string_lossy().to_string() + "/";
    if let Some(stem) = file.path.file_stem().unwrap().to_str() {
        output_path += stem;
    } else {
        output_path += file.title.as_ref().unwrap();
    }

    output_path += match out_container {
        AudioContainer::FLAC => ".flac",
        AudioContainer::MP3 => ".mp3",
        AudioContainer::M4A => ".m4a",
        AudioContainer::OGG => ".ogg",
        AudioContainer::OPUS => ".opus",
    };

    let mut ictx = format::input(&file.path)?;
    let mut octx = format::output(&output_path)?;
    let mut transcoder = transcoder(&mut ictx, &mut octx, out_codec, out_bitrate)?;

    let mut metadata = ictx.metadata().to_owned();
    if embed_cover_art {
        if let Some(cover_art) = file.ff_get_album_art().ok().flatten() {
            let mimetype: String = image::guess_format(&cover_art)
                .unwrap()
                .to_mime_type()
                .to_string();
            let block = construct_flac_picture_block(3, &mimetype, "Front cover", &cover_art);

            let cover_art_string = BASE64_STANDARD.encode(block);
            metadata.set("METADATA_BLOCK_PICTURE", &cover_art_string);
        }
    }

    octx.set_metadata(metadata);
    octx.write_header().unwrap();

    for (stream, mut packet) in ictx.packets() {
        let i = stream.index();

        if i == transcoder.stream {
            packet.rescale_ts(stream.time_base(), transcoder.in_time_base);
            transcoder.send_packet_to_decoder(&packet);
            transcoder.receive_and_process_decoded_frames(&mut octx);
            continue;
        }
    }

    transcoder.send_eof_to_decoder();
    transcoder.receive_and_process_decoded_frames(&mut octx);

    transcoder.flush_filter();
    transcoder.get_and_process_filtered_frames(&mut octx);

    transcoder.send_eof_to_encoder();
    transcoder.receive_and_process_encoded_packets(&mut octx);

    octx.write_trailer().unwrap();

    Ok(())
}

fn construct_flac_picture_block(
    pic_type: u32,
    mime: &str,
    description: &str,
    image_data: &[u8],
) -> Vec<u8> {
    let mut buf = Vec::new();

    let _ = buf.write_u32::<BigEndian>(pic_type);

    // MIME type
    let _ = buf.write_u32::<BigEndian>(mime.len() as u32).unwrap();
    let _ = buf.extend_from_slice(mime.as_bytes());

    // description
    let _ = buf
        .write_u32::<BigEndian>(description.len() as u32)
        .unwrap();
    let _ = buf.extend_from_slice(description.as_bytes());

    // unknown width, height, depth, colors
    let _ = buf.write_u32::<BigEndian>(0);
    let _ = buf.write_u32::<BigEndian>(0);
    let _ = buf.write_u32::<BigEndian>(0);
    let _ = buf.write_u32::<BigEndian>(0);

    // picture data
    let _ = buf.write_u32::<BigEndian>(image_data.len() as u32).unwrap();
    let _ = buf.extend_from_slice(image_data);

    buf
}
