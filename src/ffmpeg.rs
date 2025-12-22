use std::ptr;

use ffmpeg_next::ffi::{avcodec_alloc_context3, avcodec_find_decoder, avcodec_open2, avcodec_parameters_alloc, avcodec_parameters_to_context};

use crate::{app::AudioConverterApp, models::audio_file::AudioFile};

// NOTE: https://github.com/leandromoreira/ffmpeg-libav-tutorial <- my goat
pub fn convert_file(file: &AudioFile, settings: &AudioConverterApp) {
    let input_ctx = ffmpeg_next::format::input(&file.path).expect("Invalid path provided to FFmpeg");

    let streams = input_ctx.streams();

    for (i, stream) in streams.enumerate() {
        println!("Displaying information about stream {i}");
        println!("  Duration: {:?}", stream.duration());
        println!("  Rate: {:?}", stream.rate());
        println!("  Parameter: {:?}", stream.parameters().medium());

        // if video stream (get from stream.parameters().medium()), copy to output file

        // transcoding and transmuxing audio stream
        unsafe {
            let avcodec = avcodec_find_decoder(stream.parameters().id().into());
            let avccontext = avcodec_alloc_context3(avcodec);

            avcodec_parameters_to_context(avccontext, stream.parameters().as_ptr());
            avcodec_open2(avccontext, avcodec, ptr::null_mut());

            println!("  Codec: {:?}", *avcodec);
        }
    }
}
