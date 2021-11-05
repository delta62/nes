use ffmpeg_sys_next as ffi;
use std::ffi::{CStr, CString};
use std::ops::{Deref, DerefMut};
use std::ptr;

#[derive(Debug)]
pub enum Error {
    InitFailed,
}

pub struct SwrContext(*mut ffi::SwrContext);

impl SwrContext {
    pub fn new(src_fmt: ffi::AVSampleFormat, dst_fmt: ffi::AVSampleFormat) -> Result<Self, Error> {
        let mut ctx = not_null!(ffi::swr_alloc(), |_| Error::InitFailed)?;
        let channel_layout = ffi::AV_CH_FRONT_CENTER as i64;
        let sample_rate = 44_100;

        unsafe {
            let in_layout = CStr::from_bytes_with_nul_unchecked(b"in_channel_layout\0");
            let in_samples = CStr::from_bytes_with_nul_unchecked(b"in_sample_rate\0");
            let in_fmt = CStr::from_bytes_with_nul_unchecked(b"in_sample_fmt\0");

            ffi::av_opt_set_int(ctx as _, in_layout.as_ptr(), channel_layout, 0);
            ffi::av_opt_set_int(ctx as _, in_samples.as_ptr(), sample_rate, 0);
            ffi::av_opt_set_sample_fmt(ctx as _, in_fmt.as_ptr(), src_fmt, 0);

            let out_layout = CStr::from_bytes_with_nul_unchecked(b"out_channel_layout\0");
            let out_samples = CStr::from_bytes_with_nul_unchecked(b"out_sample_rate\0");
            let out_fmt = CStr::from_bytes_with_nul_unchecked(b"out_sample_fmt\0");

            ffi::av_opt_set_int(ctx as _, out_layout.as_ptr(), channel_layout, 0);
            ffi::av_opt_set_int(ctx as _, out_samples.as_ptr(), sample_rate, 0);
            ffi::av_opt_set_sample_fmt(ctx as _, out_fmt.as_ptr(), dst_fmt, 0);

            let ret = ffi::swr_init(ctx);

            if ret < 0 {
                ffi::swr_free(&mut ctx);
                return Err(Error::InitFailed);
            }
        }

        Ok(Self(ctx))
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::SwrContext {
        self.0
    }
}

impl Drop for SwrContext {
    fn drop(&mut self) {
        unsafe { ffi::swr_free(&mut self.0) }
    }
}

pub struct AvFormatContext(*mut ffi::AVFormatContext);

impl AvFormatContext {
    pub fn new<F>(format: F) -> Result<Self, Error>
    where F: Into<Vec<u8>> {
        let format_name = CString::new(format).unwrap();

        let mut output_ctx = ptr::null_mut();

        let ret = unsafe {
            ffi::avformat_alloc_output_context2(
                &mut output_ctx,
                ptr::null_mut(),
                format_name.as_ptr(),
                ptr::null(),
            )
        };

        if ret < 0 {
            Err(Error::InitFailed)
        } else {
            Ok(Self(output_ctx))
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::AVFormatContext {
        self.0
    }
}

impl Drop for AvFormatContext {
    fn drop(&mut self) {
        unsafe { ffi::avformat_free_context(self.0) }
    }
}

impl Deref for AvFormatContext {
    type Target = ffi::AVFormatContext;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for AvFormatContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

pub struct AvStream(*mut ffi::AVStream);

impl AvStream {
    pub fn new(context: &mut AvFormatContext) -> Result<Self, Error> {
        let ret = unsafe { ffi::avformat_new_stream(context.as_mut_ptr(), ptr::null_mut()) };

        if ret.is_null() {
            Err(Error::InitFailed)
        } else {
            Ok(Self(ret))
        }
    }
}

impl Deref for AvStream {
    type Target = ffi::AVStream;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for AvStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

pub struct AvContext(*mut ffi::AVCodecContext);

impl AvContext {
    pub fn new(codec: *const ffi::AVCodec) -> Result<Self, Error> {
        let ptr = unsafe { ffi::avcodec_alloc_context3(codec) };

        if ptr.is_null() {
            Err(Error::InitFailed)
        } else {
            Ok(Self(ptr))
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::AVCodecContext {
        self.0
    }
}

impl Drop for AvContext {
    fn drop(&mut self) {
        unsafe {
            ffi::avcodec_free_context(&mut self.0);
        }
    }
}

impl Deref for AvContext {
    type Target = ffi::AVCodecContext;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for AvContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

pub struct SwsContext(*mut ffi::SwsContext);

impl SwsContext {
    pub fn new(
        source_width: i32,
        source_height: i32,
        source_format: ffi::AVPixelFormat,
        dest_width: i32,
        dest_height: i32,
        dest_format: ffi::AVPixelFormat,
    ) -> Result<Self, Error> {
        unsafe {
            let res = ffi::sws_getContext(
                source_width,
                source_height,
                source_format,
                dest_width,
                dest_height,
                dest_format,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut()
            );

            if res.is_null() {
                Err(Error::InitFailed)
            } else {
                Ok(Self(res))
            }
        }
    }

    pub fn scale(&mut self, src_frame: &AvFrame, dst_frame: &mut AvFrame) {
        unsafe {
            ffi::sws_scale(
                self.0,
                src_frame.data.as_ptr() as *const *const u8,
                src_frame.linesize.as_ptr(),
                0,
                src_frame.height,
                dst_frame.data.as_ptr(),
                dst_frame.linesize.as_ptr(),
            );
        }
    }
}

impl Drop for SwsContext {
    fn drop(&mut self) {
        unsafe { ffi::sws_freeContext(self.0) }
    }
}

pub struct AvPacket(*mut ffi::AVPacket);

impl AvPacket {
    pub fn new() -> Result<Self, Error> {
        let ptr = unsafe { ffi::av_packet_alloc() };

        if ptr.is_null() {
            Err(Error::InitFailed)
        } else {
            Ok(Self(ptr))
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::AVPacket {
        self.0
    }
}

impl Deref for AvPacket {
    type Target = ffi::AVPacket;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for AvPacket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl Drop for AvPacket {
    fn drop(&mut self) {
        unsafe { ffi::av_packet_free(&mut self.0) }
    }
}

#[derive(Debug)]
pub struct AvFrame(*mut ffi::AVFrame);

impl AvFrame {
    pub fn new() -> Result<Self, Error> {
        let ptr = unsafe { ffi::av_frame_alloc() };

        if ptr.is_null() {
            Err(Error::InitFailed)
        } else {
            Ok(Self(ptr))
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::AVFrame {
        self.0
    }
}

impl Drop for AvFrame {
    fn drop(&mut self) {
        unsafe { ffi::av_frame_free(&mut self.0) }
    }
}

impl Deref for AvFrame {
    type Target = ffi::AVFrame;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for AvFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
