use std::env;

extern crate ffmpeg_dev;

use ffmpeg_dev::sys::{
    avformat_open_input,
    avformat_find_stream_info,
    av_dump_format,
    AVFormatContext,
};

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

    let mut ifmt_ctx: *mut AVFormatContext = std::ptr::null_mut();

    unsafe {
        avformat_open_input(&mut ifmt_ctx,
                            input_path_cstr.as_ptr(),
                            std::ptr::null_mut(),
                            std::ptr::null_mut());

        avformat_find_stream_info(ifmt_ctx, std::ptr::null_mut());

        av_dump_format(ifmt_ctx,
                       0,
                       input_path_cstr.as_ptr(),
                       0);
    }

}
