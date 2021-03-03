use gl::types::*;

use image::{DynamicImage, GenericImageView};
use std::{convert::From, ffi::c_void, path::Path};

gen_tex_builder!(TextureBuilder2D {
    (format, GLenum),
    (internal, GLenum),
    (width, i32),
    (height, i32),
    (mipmap, GLsizei),
    (min_filter, GLint),
    (mag_filter, GLint),
    (wrap_s, GLint),
    (wrap_t, GLint)
}
);

impl TextureBuilder2D {
    pub fn new(width: i32, height: i32) -> Self {
        TextureBuilder2D {
            format: gl::RGBA,
            internal: gl::RGBA8,
            width,
            height,
            mipmap: 1,
            min_filter: gl::LINEAR_MIPMAP_LINEAR as i32,
            mag_filter: gl::LINEAR as i32,
            wrap_s: gl::REPEAT as i32,
            wrap_t: gl::REPEAT as i32,
        }
    }

    pub fn empty(&self) -> Texture2D {
        let mut texture: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexStorage2D(
                gl::TEXTURE_2D,
                self.mipmap,
                self.internal,
                self.width,
                self.height,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, self.min_filter);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, self.mag_filter);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, self.wrap_s);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, self.wrap_t);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Texture2D { id: texture }
    }

    // TODO provide a alternative that validates the bytes
    pub unsafe fn with_bytes(&self, data: &[u8]) -> Texture2D {
        let texture = self.empty();

        gl::BindTexture(gl::TEXTURE_2D, texture.id);
        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            self.width,
            self.height,
            self.format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        texture
    }

    // TODO consider creating a resource manager to deal with any IO bound operation
    pub fn load_from<P: AsRef<Path>>(path: P, flip: bool) -> image::ImageResult<Texture2D> {
        let image = if flip {
            image::open(&path)?.flipv()
        } else {
            image::open(path)?
        };
        let data = image.to_bytes();

        let format = match image {
            DynamicImage::ImageBgr8(_) => gl::BGR,
            DynamicImage::ImageBgra8(_) => gl::BGRA,
            DynamicImage::ImageRgb8(_) => gl::RGB,
            DynamicImage::ImageRgba8(_) => gl::RGBA,
            DynamicImage::ImageLuma8(_) => gl::RED,
            DynamicImage::ImageLumaA8(_) => gl::RG,
            _ => unimplemented!(),
        };

        let tex = unsafe {
            Self::new(image.width() as i32, image.height() as i32)
                .format(format)
                .with_bytes(&data)
        };
        Ok(tex)
    }
}

impl Default for TextureBuilder2D {
    fn default() -> Self {
        TextureBuilder2D {
            format: gl::RGBA,
            internal: gl::RGBA8,
            width: 0,
            height: 0,
            mipmap: 1,
            min_filter: gl::LINEAR_MIPMAP_LINEAR as i32,
            mag_filter: gl::LINEAR as i32,
            wrap_s: gl::REPEAT as i32,
            wrap_t: gl::REPEAT as i32,
        }
    }
}

// (texture id, texture kind)
#[derive(Debug)]
pub struct Texture2D {
    id: GLuint,
}

impl Texture2D {
    pub fn bind(&self, slot: u32) {
        assert!(slot < gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

// TODO this should be a From, just need to finsh filling the match
impl From<gltf::image::Data> for Texture2D {
    fn from(data: gltf::image::Data) -> Self {
        use gltf::image::Format;

        let (internal, format) = match data.format {
            Format::R8 => (gl::R8, gl::RED),
            Format::R8G8 => (gl::RG8, gl::RG),
            Format::R8G8B8 => (gl::RGB8, gl::RGB),
            Format::R8G8B8A8 => (gl::RGBA8, gl::RGBA),
            Format::B8G8R8 => (gl::RGB8, gl::BGR),
            Format::B8G8R8A8 => (gl::RGBA8, gl::BGRA),
            Format::R16 => (gl::R16UI, gl::RED_INTEGER),
            Format::R16G16 => (gl::RG16UI, gl::RG_INTEGER),
            Format::R16G16B16 => (gl::RGB16UI, gl::RGB_INTEGER),
            Format::R16G16B16A16 => (gl::RGBA16UI, gl::RGBA_INTEGER),
        };
        unsafe {
            TextureBuilder2D::new(data.width as i32, data.height as i32)
                .format(format)
                .internal(internal)
                .with_bytes(&data.pixels)
        }
    }
}

impl Drop for Texture2D {
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
    fn texture_2d_builder() {
        const CONTROL: TextureBuilder2D = TextureBuilder2D {
            format: gl::RGBA,
            internal: gl::RGBA8,
            width: 0,
            height: 0,
            mipmap: 1,
            min_filter: gl::LINEAR_MIPMAP_LINEAR as i32,
            mag_filter: gl::LINEAR as i32,
            wrap_s: gl::REPEAT as i32,
            wrap_t: gl::REPEAT as i32,
        };

        let mut builder = TextureBuilder2D::new(0, 0);

        assert_eq!(CONTROL, builder);

        builder
            .format(gl::RGB)
            .internal(gl::RGB8)
            .width(500)
            .height(600)
            .mipmap(5)
            .min_filter(gl::NEAREST as i32)
            .mag_filter(gl::NEAREST as i32)
            .wrap_s(gl::CLAMP_TO_EDGE as i32)
            .wrap_t(gl::CLAMP_TO_EDGE as i32);

        assert_eq!(gl::RGB, builder.format);
        assert_eq!(gl::RGB8, builder.internal);
        assert_eq!(500, builder.width);
        assert_eq!(600, builder.height);
        assert_eq!(5, builder.mipmap);
        assert_eq!(gl::NEAREST as i32, builder.min_filter);
        assert_eq!(gl::NEAREST as i32, builder.mag_filter);
        assert_eq!(gl::CLAMP_TO_EDGE as i32, builder.wrap_s);
        assert_eq!(gl::CLAMP_TO_EDGE as i32, builder.wrap_t);
    }
}
