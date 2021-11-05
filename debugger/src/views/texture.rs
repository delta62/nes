use getset::{CopyGetters, Getters};
use imgui::TextureId;

#[derive(CopyGetters, Getters)]
pub struct Texture {
    #[getset(get_copy = "pub")]
    id: TextureId,
}

impl Texture {
    pub fn new() -> Self {
        let mut tex = 0;

        unsafe {
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let id = TextureId::new(tex as usize);
        Self { id }
    }

    pub fn update(&mut self, width: usize, height: usize, data: &[u8]) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id.id() as u32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as _,
                width as _,
                height as _,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as _
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &(self.id.id() as u32));
        }
    }
}
