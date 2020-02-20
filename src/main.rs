use std::env;
use std::result::Result;

extern crate ffmpeg_dev;

use ffmpeg_dev::sys::{
    avformat_open_input,
    avformat_close_input,
    avformat_find_stream_info,
    av_dump_format,
    avcodec_register_all,
    avcodec_alloc_context3,
    avcodec_free_context,
    avcodec_parameters_to_context,
    avcodec_find_decoder,
    avcodec_open2,
    avcodec_close,
    AVFormatContext,
    AVCodecContext,
    AVStream,
};

use ffmpeg_dev::sys::{
    AVMediaType_AVMEDIA_TYPE_VIDEO as AVMEDIA_TYPE_VIDEO,
};

mod process;

fn get_video_stream<'a>(streams: &'a Vec<&AVStream>) -> Result<(&'a AVStream, usize), &'static str> {
    unsafe {
        for (index, stream_ptr) in streams.iter().enumerate() {
            let codecpar = *stream_ptr.codecpar;
            if codecpar.codec_type == AVMEDIA_TYPE_VIDEO {
                println!("found video stream at index {}", index);
                return Ok((streams[index], index));
            }
        }
    }

    return Err("no video stream found");
}

fn get_streams(ifmt_ctx: &AVFormatContext) -> Vec<&AVStream> {
    let nb_streams = (*ifmt_ctx).nb_streams as usize;
    unsafe {
        let streams = std::slice::from_raw_parts((*ifmt_ctx).streams, nb_streams)
            .iter()
            .map(|x| (*x).as_ref().expect("not null"))
            .collect::<Vec<&AVStream>>();

        streams
    }

}

fn main() {
    println!("starting timer");

    let args: Vec<_> = env::args().collect();
    let uri: &str = if args.len() == 2 {
        args[1].as_ref()
    } else {
        eprintln!("usage: h2a-timer <filename>");
        std::process::exit(-1);
    };

    println!("got arg: {}", uri);

    let input_path_cstr = match std::ffi::CString::new(uri) {
        Ok(s) => s,
        Err(e) => panic!(e),
    };

    let mut ifmt_ctx_ptr: *mut AVFormatContext = unsafe {
        let mut ifmt_ctx_ptr: *mut AVFormatContext = std::ptr::null_mut();
        avformat_open_input(&mut ifmt_ctx_ptr,
                            input_path_cstr.as_ptr(),
                            std::ptr::null_mut(),
                            std::ptr::null_mut());

        if ifmt_ctx_ptr == std::ptr::null_mut() {
            eprintln!("failed to get format context");
            std::process::exit(-1);
        }

        avformat_find_stream_info(ifmt_ctx_ptr, std::ptr::null_mut());

        av_dump_format(ifmt_ctx_ptr,
                       0,
                       input_path_cstr.as_ptr(),
                       0);

        ifmt_ctx_ptr
    };

    let mut ifmt_ctx: &mut AVFormatContext = unsafe { &mut *ifmt_ctx_ptr };

    let streams: Vec<&AVStream> = get_streams(&ifmt_ctx);

    let (video_stream, video_stream_idx) = match get_video_stream(&streams) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(-1);
        }
    };

    let mut codec_ctx_ptr: *mut AVCodecContext = unsafe {
        avcodec_register_all();

        let codec_ctx_ptr = avcodec_alloc_context3(std::ptr::null_mut());
        avcodec_parameters_to_context(codec_ctx_ptr, video_stream.codecpar);
        if codec_ctx_ptr == std::ptr::null_mut() {
            eprintln!("failed to get codec from codec parameters");
            std::process::exit(-1);
        }

        println!("got codec parameters");

        let codec_ptr = avcodec_find_decoder((*codec_ctx_ptr).codec_id);
        if codec_ptr == std::ptr::null_mut() {
            eprintln!("codec not supported: id:{}", (*codec_ctx_ptr).codec_id);
            std::process::exit(-1);
        }

        let codec = *codec_ptr;

        avcodec_open2(codec_ctx_ptr, &codec, std::ptr::null_mut());

        codec_ctx_ptr
    };

    let mut codec_ctx: &mut AVCodecContext = unsafe { &mut *codec_ctx_ptr };

    let intervals: Vec<(u32, u32)> = process::find_white_frame_intervals(&mut ifmt_ctx, &mut codec_ctx, video_stream_idx);

    unsafe {
        avcodec_close(codec_ctx_ptr);
        avformat_close_input(&mut ifmt_ctx_ptr);
        avcodec_free_context(&mut codec_ctx_ptr);
    }

    println!("done");

}
