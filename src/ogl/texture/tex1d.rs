use gl::types::*;

gen_tex_builder!(TextureBuilder1D {
    (length, GLsizei),
    (mipmap, GLsizei),
    (internal_format, GLenum),
    (data_format, GLenum)
});

impl TextureBuilder1D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty(&self) -> Texture1D {
        let mut texture: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_1D, texture);
            gl::TexStorage1D(
                gl::TEXTURE_1D,
                self.mipmap,
                self.internal_format,
                self.length,
            );

            // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, self.min_filter);
            // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, self.mag_filter);
            // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, self.wrap_s);
            // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, self.wrap_t);

            gl::BindTexture(gl::TEXTURE_2D, 0);

            Texture1D { id: texture }
        }
    }

    pub unsafe fn with_bytes(&self, data: &[u8]) -> Texture1D {
        let texture = self.empty();

        gl::BindTexture(gl::TEXTURE_1D, texture.id);
        gl::TexSubImage1D(
            gl::TEXTURE_1D,
            0,
            0,
            self.length,
            self.data_format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const std::ffi::c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_1D);
        gl::BindTexture(gl::TEXTURE_1D, 0);

        texture
    }
}

impl Default for TextureBuilder1D {
    fn default() -> Self {
        TextureBuilder1D {
            length: 0,
            mipmap: 1,
            internal_format: gl::RGBA8,
            data_format: gl::RGBA,
        }
    }
}

#[derive(Debug)]
pub struct Texture1D {
    id: GLuint,
}

impl Texture1D {
    pub fn bind(&self, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_1D, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_1D, 0);
        }
    }
}

impl Drop for Texture1D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_1d_builder() {
        const CONTROL: TextureBuilder1D = TextureBuilder1D {
            length: 0,
            mipmap: 1,
            internal_format: gl::RGBA8,
            data_format: gl::RGBA,
        };

        let mut builder = TextureBuilder1D::new();

        assert_eq!(CONTROL, builder);

        builder
            .length(12)
            .mipmap(3)
            .internal_format(gl::RGB8)
            .data_format(gl::RGB);

        assert_eq!(builder.length, 12);
        assert_eq!(builder.mipmap, 3);
        assert_eq!(builder.internal_format, gl::RGB8);
        assert_eq!(builder.data_format, gl::RGB);
    }
}
