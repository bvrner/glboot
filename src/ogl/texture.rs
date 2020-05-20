use gl::types::*;

use image::{DynamicImage, GenericImageView};
use std::{convert::TryInto, ffi::c_void, path::Path};

#[derive(Debug)]
pub struct Texture(GLuint);

impl Texture {
    pub fn new<P: AsRef<Path>>(path: P, flip: bool) -> image::ImageResult<Texture> {
        let image = if flip {
            image::open(&path)?.flipv()
        } else {
            image::open(path)?
        };
        let data = image.to_bytes();

        let (internal_format, format) = match image {
            DynamicImage::ImageBgr8(_) => (gl::RGB, gl::BGR),
            DynamicImage::ImageBgra8(_) => (gl::RGBA, gl::BGRA),
            DynamicImage::ImageRgb8(_) => (gl::RGB, gl::RGB),
            DynamicImage::ImageRgba8(_) => (gl::RGBA, gl::RGBA),
            DynamicImage::ImageLuma8(_) => (gl::RED, gl::RED),
            DynamicImage::ImageLumaA8(_) => (gl::RG, gl::RG),
            _ => unimplemented!(),
        };

        unsafe {
            let mut texture: GLuint = 0;

            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format.try_into().unwrap(),
                image.width() as i32,
                image.height() as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                &data[0] as *const u8 as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);

            Ok(Texture(texture))
        }
    }

    pub fn bind(&self, slot: u32) {
        assert!(slot < gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, self.0);
        }
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.0);
        }
    }
}
