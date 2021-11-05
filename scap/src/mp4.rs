use crate::ffmpeg::*;
use ffmpeg_sys_next as ffi;
use nix::errno::Errno;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::ptr;

const AUDIO_CHANNELS: i32 = 1;

struct AudioEncoder {
    ctx: AvContext,
    frame: AvFrame,
    next_pts: i64,
    stream: AvStream,
}

impl AudioEncoder {
    fn new(config: &Mp4Config, output_ctx: &mut AvFormatContext) -> Result<Self, Error> {
        let oformat = unsafe { *output_ctx.oformat };
        let codec_id = oformat.audio_codec;
        let codec = not_null!(ffi::avcodec_find_encoder(codec_id), |_| Error::InitFailed)?;

        let mut ctx = AvContext::new(codec)?;
        ctx.bit_rate = config.audio_bit_rate;
        ctx.channel_layout = ffi::AV_CH_LAYOUT_MONO;
        ctx.channels = AUDIO_CHANNELS;
        ctx.codec_id = codec_id;
        ctx.sample_fmt = ffi::AVSampleFormat::AV_SAMPLE_FMT_FLT;
        ctx.sample_rate = config.audio_sample_rate;

        if mask!(oformat.flags, ffi::AVFMT_GLOBALHEADER) {
            ctx.flags |= ffi::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
        }

        // open audio
        not_negative!(
            ffi::avcodec_open2(ctx.as_mut_ptr(), codec, ptr::null_mut()),
            |_| Error::InitFailed,
        )?;

        not_negative!(
            ffi::avcodec_open2(ctx.as_mut_ptr(), codec, ptr::null_mut()),
            |_| Error::InitFailed,
        )?;

        let mut stream = AvStream::new(output_ctx)?;
        stream.id = output_ctx.nb_streams as i32 - 1;
        stream.time_base = ffi::AVRational { num: 1, den: config.audio_sample_rate };

        not_negative!(
            ffi::avcodec_parameters_from_context(stream.codecpar, ctx.as_mut_ptr()),
            |_| Error::InitFailed,
        )?;

        let mut frame = AvFrame::new()?;
        frame.format = ffi::AVSampleFormat::AV_SAMPLE_FMT_FLT as i32;
        frame.channel_layout = ctx.channel_layout;
        frame.sample_rate = ctx.sample_rate;
        frame.nb_samples = ctx.frame_size;

        not_negative!(
            ffi::av_frame_get_buffer(frame.as_mut_ptr(), 0),
            |_| Error::InitFailed,
        )?;

        let next_pts = 0;

        Ok(Self { ctx, frame, next_pts, stream })
    }

    fn samples_per_frame(&self) -> usize {
        self.frame.nb_samples as usize
    }

    fn prepare_frame(&mut self, frame: &[f32]) -> Result<(), Error> {
        not_negative!(
            ffi::av_frame_make_writable(self.frame.as_mut_ptr()),
            |_| Error::InitFailed,
        )?;

        let dest = self.frame.data[0] as *mut f32;
        for idx in 0..frame.len() {
            let sample = frame[idx as usize];

            unsafe {
                let dest = dest.offset(idx as isize);
                *dest = sample;
            }
        }

        self.frame.pts = unsafe {
            ffi::av_rescale_q(
                self.next_pts,
                ffi::AVRational { num: 1, den: self.frame.sample_rate },
                self.ctx.time_base,
            )
        };
        self.next_pts += frame.len() as i64;

        Ok(())
    }

    fn encode_frame<F>(&mut self, frame: &[f32], mut enc_cb: F) -> Result<(), Error>
    where F: FnMut(&mut AvContext, Option<&mut AvFrame>, &AvStream) -> Result<(), Error> {
        self.prepare_frame(frame)?;
        enc_cb(&mut self.ctx, Some(&mut self.frame), &self.stream)
    }

    fn flush<F>(&mut self, mut enc_cb: F) -> Result<(), Error>
    where F: FnMut(&mut AvContext, Option<&mut AvFrame>, &AvStream) -> Result<(), Error> {
        enc_cb(&mut self.ctx, None, &self.stream)
    }
}

struct VideoEncoder {
    ctx: AvContext,
    next_pts: i64,
    rgb_frame: AvFrame,
    stream: AvStream,
    sws_context: SwsContext,
    yuv_frame: AvFrame,
}

impl VideoEncoder {
    fn new(config: &Mp4Config, output_ctx: &mut AvFormatContext) -> Result<Self, Error> {
        let oformat = unsafe { *output_ctx.oformat };
        let codec_id = oformat.video_codec;
        let codec = not_null!(ffi::avcodec_find_encoder(codec_id), |_| Error::InitFailed)?;

        let mut stream = AvStream::new(output_ctx)?;
        stream.id = output_ctx.nb_streams as i32 - 1;
        stream.time_base = ffi::AVRational { num: 1, den: 60 };

        let mut ctx = AvContext::new(codec)?;
        ctx.codec_id = codec_id;
        ctx.bit_rate = config.video_bit_rate;
        ctx.width = config.output_width;
        ctx.height = config.output_height;
        ctx.time_base = ffi::AVRational { num: 1, den: 60 };
        ctx.gop_size = 12;
        ctx.pix_fmt = ffi::AVPixelFormat::AV_PIX_FMT_YUV420P;

        if mask!(oformat.flags, ffi::AVFMT_GLOBALHEADER) {
            ctx.flags |= ffi::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
        }

        unsafe {
            let key = CStr::from_bytes_with_nul_unchecked(b"preset\0");
            let val = CStr::from_bytes_with_nul_unchecked(b"fast\0");
            ffi::av_opt_set(ctx.priv_data, key.as_ptr(), val.as_ptr(), 0);
        }

        not_negative!(
            ffi::avcodec_open2(ctx.as_mut_ptr(), codec, ptr::null_mut()),
            |_| Error::InitFailed
        )?;

        not_negative!(
            ffi::avcodec_parameters_from_context(stream.codecpar, ctx.as_mut_ptr()),
            |_| Error::InitFailed,
        )?;

        let mut rgb_frame = AvFrame::new()?;
        rgb_frame.format = ffi::AVPixelFormat::AV_PIX_FMT_RGB24 as i32;
        rgb_frame.width = config.input_width;
        rgb_frame.height = config.input_height;

        not_negative!(ffi::av_frame_get_buffer(rgb_frame.as_mut_ptr(), 0), |_| Error::InitFailed)?;

        let mut yuv_frame = AvFrame::new()?;
        yuv_frame.format = ffi::AVPixelFormat::AV_PIX_FMT_YUV420P as i32;
        yuv_frame.width = config.output_width;
        yuv_frame.height = config.output_height;

        not_negative!(ffi::av_frame_get_buffer(yuv_frame.as_mut_ptr(), 0), |_| Error::InitFailed)?;

        let sws_context = SwsContext::new(
            config.input_width,
            config.input_height,
            ffi::AVPixelFormat::AV_PIX_FMT_RGB24,
            config.output_width,
            config.output_height,
            ffi::AVPixelFormat::AV_PIX_FMT_YUV420P,
        )?;

        let next_pts = 0;

        Ok(Self { ctx, next_pts, rgb_frame, stream, sws_context, yuv_frame })
    }

    fn prepare_frame(&mut self, frame: &[u8]) -> Result<(), Error> {
        not_negative!(
            ffi::av_frame_make_writable(self.yuv_frame.as_mut_ptr()),
            |_| Error::InitFailed,
        )?;

        // fill rgb image
        let dest = self.rgb_frame.data[0] as *mut u8;
        for y in 0..self.rgb_frame.height {
            for x in 0..self.rgb_frame.height {
                let offset_i = (y * 256 * 3 + x * 3) as isize;
                let offset_u = offset_i as usize;

                unsafe {
                    let dest_r = dest.offset(offset_i + 0);
                    let dest_g = dest.offset(offset_i + 1);
                    let dest_b = dest.offset(offset_i + 2);

                    *dest_r = frame[offset_u + 0];
                    *dest_g = frame[offset_u + 1];
                    *dest_b = frame[offset_u + 2];
                }
            }
        }

        self.sws_context.scale(&self.rgb_frame, &mut self.yuv_frame);

        self.yuv_frame.pts = self.next_pts;
        self.next_pts += 1;

        Ok(())
    }

    fn encode_frame<F>(&mut self, frame: &[u8], mut enc_cb: F) -> Result<(), Error>
    where F: FnMut(&mut AvContext, Option<&mut AvFrame>, &AvStream) -> Result<(), Error> {
        self.prepare_frame(frame)?;
        enc_cb(&mut self.ctx, Some(&mut self.yuv_frame), &self.stream)
    }

    fn flush<F>(&mut self, mut enc_cb: F) -> Result<(), Error>
    where F: FnMut(&mut AvContext, Option<&mut AvFrame>, &AvStream) -> Result<(), Error> {
        enc_cb(&mut self.ctx, None, &self.stream)
    }
}

pub struct Mp4Config {
    pub input_width: i32,
    pub input_height: i32,
    pub output_width: i32,
    pub output_height: i32,
    pub video_bit_rate: i64,
    pub audio_bit_rate: i64,
    pub audio_sample_rate: i32,
}

pub struct Mp4Encoder {
    audio_encoder: AudioEncoder,
    leftover_audio: VecDeque<f32>,
    output_ctx: AvFormatContext,
    video_encoder: VideoEncoder,
}

impl Mp4Encoder {
    pub fn new<P>(path: P, config: &Mp4Config) -> Result<Self, Error>
    where P: Into<Vec<u8>> {
        unsafe { ffi::av_log_set_level(ffi::AV_LOG_WARNING) };

        let filename = CString::new(path).unwrap();
        let mut output_ctx = AvFormatContext::new("mp4")?;
        let video_encoder = VideoEncoder::new(config, &mut output_ctx)?;
        let audio_encoder = AudioEncoder::new(config, &mut output_ctx)?;
        let leftover_audio = VecDeque::with_capacity(audio_encoder.samples_per_frame());

        not_negative!(
            ffi::avio_open(&mut output_ctx.pb, filename.as_ptr(), ffi::AVIO_FLAG_WRITE),
            |_| Error::InitFailed,
        )?;

        not_negative!(
            ffi::avformat_write_header(output_ctx.as_mut_ptr(), ptr::null_mut()),
            |_| Error::InitFailed,
        )?;

        Ok(Self { audio_encoder, leftover_audio, output_ctx, video_encoder })
    }

    pub fn encode_frame(&mut self, rgb_frame: &[u8], audio_samples: &[f32]) -> Result<(), Error> {
        let mut have_video = true;

        // Buffer all the audio
        for sample in audio_samples {
            self.leftover_audio.push_back(*sample);
        }

        let samples_per_frame = self.audio_encoder.samples_per_frame();
        let mut have_audio = self.leftover_audio.len() >= samples_per_frame;

        while have_video || have_audio {
            if have_video {
                let output_ctx = &mut self.output_ctx;
                self.video_encoder.encode_frame(rgb_frame, |ctx, frame, stream| {
                    Self::encode_ffmpeg_frame(ctx, frame, stream, output_ctx)
                })?;

                have_video = false;
            } else {
                let mut frame = Vec::with_capacity(samples_per_frame);
                for _ in 0..samples_per_frame {
                    frame.push(self.leftover_audio.pop_front().unwrap());
                }

                let output_ctx = &mut self.output_ctx;
                self.audio_encoder.encode_frame(&frame, |ctx, frame, stream| {
                    Self::encode_ffmpeg_frame(ctx, frame, stream, output_ctx)
                })?;

                have_audio = self.leftover_audio.len() >= samples_per_frame;
            }
        }

        Ok(())
    }

    fn encode_ffmpeg_frame(
        ctx: &mut AvContext,
        frame: Option<&mut AvFrame>,
        stream: &AvStream,
        output_ctx: &mut AvFormatContext,
    ) -> Result<(), Error> {
        let frame_ptr = match frame {
            Some(frame) => frame.as_mut_ptr(),
            None        => ptr::null_mut(),
        };

        not_negative!(
            ffi::avcodec_send_frame(ctx.as_mut_ptr(), frame_ptr),
            |_| Error::InitFailed,
        )?;

        loop {
            let mut packet = AvPacket::new().unwrap();

            let recv_result = unsafe {
                ffi::avcodec_receive_packet(ctx.as_mut_ptr(), packet.as_mut_ptr())
            };

            let errno = Errno::from_i32(-recv_result);
            if errno == Errno::EAGAIN || recv_result == ffi::AVERROR_EOF {
                return Ok(())
            } else if recv_result < 0 {
                return Err(Error::InitFailed);
            }

            packet.stream_index = stream.id;
            unsafe {
                ffi::av_packet_rescale_ts(packet.as_mut_ptr(), ctx.time_base, stream.time_base);
            }

            let output_result = unsafe {
                ffi::av_interleaved_write_frame(
                    output_ctx.as_mut_ptr(),
                    packet.as_mut_ptr()
                )
            };

            if output_result != 0 {
                return Err(Error::InitFailed);
            }
        }
    }

    pub fn finish(mut self) -> Result<(), Error> {
        // send remaining audio samples if any
        let len = self.leftover_audio.len();
        let mut frame = Vec::with_capacity(len);
        for _ in 0..len {
            frame.push(self.leftover_audio.pop_front().unwrap());
        }

        let output_ctx = &mut self.output_ctx;
        self.audio_encoder.encode_frame(&frame, |ctx, frame, stream| {
            Self::encode_ffmpeg_frame(ctx, frame, stream, output_ctx)
        })?;

        // Flush video encoder
        self.video_encoder.flush(|ctx, frame, stream| {
            Self::encode_ffmpeg_frame(ctx, frame, stream, output_ctx)
        })?;

        // Flush audio encoder
        self.audio_encoder.flush(|ctx, frame, stream| {
            Self::encode_ffmpeg_frame(ctx, frame, stream, output_ctx)
        })?;

        // Flush the interleaved frame buffer
        not_negative!(
            ffi::av_interleaved_write_frame(self.output_ctx.as_mut_ptr(), ptr::null_mut()),
            |_| Error::InitFailed,
        )?;

        not_negative!(
            ffi::av_write_trailer(self.output_ctx.as_mut_ptr()),
            |_| Error::InitFailed,
        )?;

        unsafe { ffi::avio_closep(&mut self.output_ctx.pb) };

        Ok(())
    }
}
