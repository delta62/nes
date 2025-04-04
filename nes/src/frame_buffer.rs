use std::ops::{Index, IndexMut};

const SCREEN_BYTES_RGB: usize = 256 * 240 * 3;

#[derive(Debug, Clone)]
pub struct Frame {
    frame: Box<[u8; SCREEN_BYTES_RGB]>,
}

impl Frame {
    pub fn new() -> Self {
        let frame = Box::new([0; SCREEN_BYTES_RGB]);
        Self { frame }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u8; SCREEN_BYTES_RGB]> for Frame {
    fn as_ref(&self) -> &[u8; SCREEN_BYTES_RGB] {
        self.frame.as_ref()
    }
}

impl Index<usize> for Frame {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.frame[index]
    }
}

impl IndexMut<usize> for Frame {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.frame[index]
    }
}

pub struct FrameBuffer {
    frames: Vec<Frame>,
}

impl FrameBuffer {
    pub fn new() -> Self {
        let frames = vec![Frame::new(), Frame::new(), Frame::new()];
        Self { frames }
    }

    pub fn put(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn get(&mut self) -> Frame {
        match self.frames.pop() {
            Some(frame) => frame,
            None => Frame::new(),
        }
    }
}
