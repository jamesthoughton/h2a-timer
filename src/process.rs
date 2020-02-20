extern crate ffmpeg_dev;

use std::result::Result;

use ffmpeg_dev::sys::{
    av_read_frame,
    av_init_packet,
    av_packet_unref,
    avcodec_send_packet,
    avcodec_receive_frame,
    av_frame_alloc,
    av_frame_free,
    av_frame_unref,
    AVFrame,
    AVPacket,
    AVCodecContext,
    AVFormatContext,
    EOF,
    EAGAIN,
};

use ffmpeg_dev::extra::defs::{
    averror
};

static AV_EAGAIN: i32 = EAGAIN as i32;

fn make_av_packet() -> AVPacket {
    let mut packet = AVPacket {
        buf: std::ptr::null_mut(),
        pts: 0,
        dts: 0,
        data: std::ptr::null_mut(),
        size: 0,
        stream_index: 0,
        flags: 0,
        side_data: std::ptr::null_mut(),
        side_data_elems: 0,
        duration: 0,
        pos: 0,
        convergence_duration: 0
    };

    unsafe { av_init_packet(&mut packet); }

    packet
}

fn process_frame(codec_ctx: &mut AVCodecContext, packet: &mut AVPacket, frame: &mut AVFrame) -> Result<bool, (&'static str, i32)> {
    let mut ret: i32;
    if {
        unsafe { ret = avcodec_send_packet(codec_ctx, packet) };
        ret != 0
    } {
        return Err(("avcodec_send_packet", ret));
    }

    while {
        ret = unsafe { avcodec_receive_frame(codec_ctx, frame) };
        if ret != 0 && ret != EOF && ret != unsafe { averror(AV_EAGAIN) } {
            return Err(("avcodec_receive_frame", ret));
        }
        ret == AV_EAGAIN
    } { }


    unsafe {
        av_frame_unref(frame);
    }

    return Ok(false);
}

pub fn find_white_frame_intervals(fmt_ctx: &mut AVFormatContext, mut codec_ctx: &mut AVCodecContext, video_stream_idx: usize) -> Vec<(u32, u32)> {
    println!("searching for white frame intervals");

    let mut vec = Vec::new();

    let mut packet: AVPacket = make_av_packet();

    let mut av_frame_ptr: *mut AVFrame = unsafe { av_frame_alloc() };
    let mut av_frame: &mut AVFrame = unsafe { &mut *av_frame_ptr };

    println!("initialized packet");

    let mut frame: u32 = 0;
    while unsafe { av_read_frame(fmt_ctx, &mut packet) } >= 0
    {
        if video_stream_idx as i32 != packet.stream_index {
            continue;
        }

        print!("\rProcessing frame: {}", frame);
        frame += 1;

        let white: bool = match process_frame(&mut codec_ctx, &mut packet, &mut av_frame) {
            Err((f, c)) => panic!("unhandled error returned from {}: code {}", f, c),
            Ok(b) => b,
        };

        unsafe {
            av_packet_unref(&mut packet);
        }
    }
    println!();

    unsafe {
        av_frame_free(&mut av_frame_ptr);
    }

    println!("done with frames");

    return vec;
}
