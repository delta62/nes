use crate::ffmpeg::*;
use ffmpeg_sys_next as ffi;
use nix::errno::Errno;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::ptr;
use std::slice;

const AUDIO_CHANNELS: i32 = 1;

pub struct WavConfig {
    pub sample_rate: i32,
}

pub struct WavEncoder {
    buffer: VecDeque<f32>,
    ctx: AvContext,
    frame: AvFrame,
    packet: AvPacket,
    swr_ctx: SwrContext,
    tmp_frame: AvFrame,
    writer: BufWriter<File>,
}

impl WavEncoder {
    pub fn new<P>(path: P, config: &WavConfig) -> Result<Self, Error>
    where P: AsRef<Path> {
        let codec_id = ffi::AVCodecID::AV_CODEC_ID_WAVPACK;
        let codec = not_null!(ffi::avcodec_find_encoder(codec_id), |_| Error::InitFailed)?;

        let mut ctx = AvContext::new(codec)?;
        ctx.channel_layout = ffi::AV_CH_LAYOUT_MONO;
        ctx.channels = AUDIO_CHANNELS;
        ctx.codec_id = codec_id;
        ctx.sample_fmt = ffi::AVSampleFormat::AV_SAMPLE_FMT_S16;
        ctx.sample_rate = config.sample_rate;

        not_negative!(
            ffi::avcodec_open2(ctx.as_mut_ptr(), codec, ptr::null_mut()),
            |_| Error::InitFailed,
        )?;

        let mut tmp_frame = AvFrame::new()?;
        tmp_frame.format = ffi::AVSampleFormat::AV_SAMPLE_FMT_FLT as i32;
        tmp_frame.channel_layout = ctx.channel_layout;
        tmp_frame.sample_rate = ctx.sample_rate;
        tmp_frame.nb_samples = ctx.frame_size;

        not_negative!(
            ffi::av_frame_get_buffer(tmp_frame.as_mut_ptr(), 0),
            |_| Error::InitFailed,
        )?;

        let mut frame = AvFrame::new()?;
        frame.format = ffi::AVSampleFormat::AV_SAMPLE_FMT_S16 as i32;
        frame.channel_layout = ctx.channel_layout;
        frame.sample_rate = ctx.sample_rate;
        frame.nb_samples = ctx.frame_size;

        not_negative!(
            ffi::av_frame_get_buffer(frame.as_mut_ptr(), 0),
            |_| Error::InitFailed,
        )?;

        let swr_ctx = SwrContext::new(
            ffi::AVSampleFormat::AV_SAMPLE_FMT_FLT,
            ffi::AVSampleFormat::AV_SAMPLE_FMT_S16,
        )?;

        let packet = AvPacket::new()?;
        let buffer = VecDeque::with_capacity(ctx.frame_size as usize);

        let file = File::create(path).map_err(|_| Error::InitFailed)?;
        let writer = BufWriter::new(file);

        Ok(Self { buffer, frame, ctx, packet, swr_ctx, tmp_frame, writer })
    }

    fn prepare_frame(&mut self) -> Result<(), Error> {
        not_negative!(
            ffi::av_frame_make_writable(self.frame.as_mut_ptr()),
            |_| Error::InitFailed,
        )?;

        let frame_size = self.ctx.frame_size as isize;
        let dest = self.tmp_frame.data[0] as *mut f32;

        for idx in 0..frame_size {
            let sample = self.buffer.pop_front().unwrap();

            unsafe {
                let dest = dest.offset(idx);
                *dest = sample;
            }
        }

        not_negative!(
            ffi::swr_convert(
                self.swr_ctx.as_mut_ptr(),
                self.frame.data.as_mut_ptr(),
                frame_size as _,
                self.tmp_frame.data.as_mut_ptr() as _,
                frame_size as _,
            ),
            |_| Error::InitFailed,
        )?;

        Ok(())
    }

    pub fn encode(&mut self, data: &mut VecDeque<f32>) -> Result<(), Error> {
        for sample in data {
            self.buffer.push_back(*sample);
        }

        self.encode_frame(false)
    }

    fn encode_frame(&mut self, flush: bool) -> Result<(), Error> {
        let frame: *mut ffi::AVFrame;

        if flush {
            frame = ptr::null_mut();
        } else {
            let have_enough_data = self.buffer.len() >= self.ctx.frame_size as usize;
            if !have_enough_data {
                return Ok(())
            }

            self.prepare_frame()?;
            frame = self.frame.as_mut_ptr();
        }

        not_negative!(
            ffi::avcodec_send_frame(self.ctx.as_mut_ptr(), frame),
            |_| Error::InitFailed,
        )?;

        loop {
            let ret = unsafe {
                ffi::avcodec_receive_packet(self.ctx.as_mut_ptr(), self.packet.as_mut_ptr())
            };

            let err = Errno::from_i32(-ret);
            if err == Errno::EAGAIN || ret == ffi::AVERROR_EOF {
                break;
            }

            if ret < 0 {
                return Err(Error::InitFailed);
            }

            let data = unsafe {slice::from_raw_parts(self.packet.data, self.packet.size as usize) };
            self.writer.write(data).map_err(|_| Error::InitFailed)?;

            unsafe { ffi::av_packet_unref(self.packet.as_mut_ptr()) }
        }

        Ok(())
    }

    pub fn finish(mut self) -> Result<(), Error> {
        self.encode_frame(true)
    }
}
