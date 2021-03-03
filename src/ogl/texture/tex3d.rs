use gl::types::*;

use std::ffi::c_void;

gen_tex_builder!(TextureBuilder3D {
    (data_format, GLenum),
    (internal_format, GLenum),
    (width, i32),
    (height, i32),
    (depth, i32),
    (mipmap, GLsizei)
    // (min_filter, GLint), // not currently needed
    // (mag_filter, GLint),
    // (wrap_s, GLint),
    // (wrap_t, GLint)
}
);

impl TextureBuilder3D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty(&self) -> Texture3D {
        let mut texture: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_3D, texture);
            gl::TexStorage3D(
                gl::TEXTURE_2D,
                self.mipmap,
                self.internal_format,
                self.width,
                self.height,
                self.depth,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Texture3D { id: texture }
    }

    pub unsafe fn with_bytes(&self, data: &[u8]) -> Texture3D {
        let texture = self.empty();

        gl::BindTexture(gl::TEXTURE_3D, texture.id);
        gl::TexSubImage3D(
            gl::TEXTURE_3D,
            0,
            0,
            0,
            0,
            self.width,
            self.height,
            self.depth,
            self.data_format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_3D);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        texture
    }
}

impl Default for TextureBuilder3D {
    fn default() -> Self {
        TextureBuilder3D {
            data_format: gl::RGBA,
            internal_format: gl::RGBA8,
            width: 0,
            height: 0,
            depth: 0,
            mipmap: 1,
            // min_filter: gl::LINEAR_MIPMAP_LINEAR as i32,
            // mag_filter: gl::LINEAR as i32,
            // wrap_s: gl::REPEAT as i32,
            // wrap_t: gl::REPEAT as i32,
        }
    }
}

// (texture id, texture kind)
#[derive(Debug)]
pub struct Texture3D {
    id: GLuint,
}

impl Texture3D {
    pub fn bind(&self, slot: u32) {
        assert!(slot < gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_3D, self.id);
        }
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_3D, 0);
        }
    }
}

impl Drop for Texture3D {
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
    fn texture_3d_builder() {
        const CONTROL: TextureBuilder3D = TextureBuilder3D {
            data_format: gl::RGBA,
            internal_format: gl::RGBA8,
            width: 0,
            height: 0,
            depth: 0,
            mipmap: 1,
            // min_filter: gl::LINEAR_MIPMAP_LINEAR as i32,
            // mag_filter: gl::LINEAR as i32,
            // wrap_s: gl::REPEAT as i32,
            // wrap_t: gl::REPEAT as i32,
        };

        let mut builder = TextureBuilder3D::new();

        assert_eq!(CONTROL, builder);

        builder
            .width(30)
            .height(45)
            .depth(27)
            .mipmap(3)
            .internal_format(gl::RGB8)
            .data_format(gl::RGB);

        assert_eq!(builder.width, 30);
        assert_eq!(builder.height, 45);
        assert_eq!(builder.depth, 27);
        assert_eq!(builder.mipmap, 3);
        assert_eq!(builder.internal_format, gl::RGB8);
        assert_eq!(builder.data_format, gl::RGB);
    }
}
