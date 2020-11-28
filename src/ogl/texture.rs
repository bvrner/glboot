use gl::types::*;

use image::{DynamicImage, GenericImageView};
use std::convert::TryFrom;
use std::{convert::TryInto, ffi::c_void, path::Path};

// (texture id, texture kind)
#[derive(Debug)]
pub struct Texture(GLuint, GLuint);

impl Texture {
    /// Crates a new texture from a file in `path`, and optionally vertically flips the resulting texture.
    pub fn new<P: AsRef<Path>>(path: P, flip: bool) -> image::ImageResult<Texture> {
        let image = if flip {
            image::open(&path)?.flipv()
        } else {
            image::open(path)?
        };
        let data = image.to_bytes();

        let (_internal_format, format) = match image {
            DynamicImage::ImageBgr8(_) => (gl::RGB, gl::BGR),
            DynamicImage::ImageBgra8(_) => (gl::RGBA, gl::BGRA),
            DynamicImage::ImageRgb8(_) => (gl::RGB, gl::RGB),
            DynamicImage::ImageRgba8(_) => (gl::RGBA, gl::RGBA),
            DynamicImage::ImageLuma8(_) => (gl::RED, gl::RED),
            DynamicImage::ImageLumaA8(_) => (gl::RG, gl::RG),
            _ => unimplemented!(),
        };

        unsafe {
            Ok(Self::from_bytes(
                &data,
                image.width() as i32,
                image.height() as i32,
                format,
            ))
        }
    }

    /// Creates a new texture from it's raw bytes, with the specified width, height and OpenGL's `format`.
    /// ## Safety
    /// Ill combinations of formats and/or dimensions can result in a segmentation fault.
    pub unsafe fn from_bytes(data: &[u8], width: i32, height: i32, format: GLenum) -> Self {
        let mut texture: GLuint = 0;

        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            format.try_into().unwrap(),
            width,
            height,
            0,
            format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const c_void,
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

        Texture(texture, gl::TEXTURE_2D)
    }

    /// Create a cubemap from a list of paths to the textures.
    /// This assumes a specific order for the paths: right, left, top, bottom, front and back.
    pub fn cubemap<P: AsRef<Path>>(paths: &[P], flip: bool) -> image::ImageResult<Texture> {
        unsafe {
            let mut texture: GLuint = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);

            for (i, path) in paths.iter().enumerate() {
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

                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    internal_format as i32,
                    image.width() as i32,
                    image.height() as i32,
                    0,
                    format,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr() as *const c_void,
                );
            }
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as i32,
            );

            Ok(Texture(texture, gl::TEXTURE_CUBE_MAP))
        }
    }

    pub fn bind(&self, slot: u32) {
        assert!(slot < gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(self.1, self.0);
        }
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl TryFrom<gltf::image::Data> for Texture {
    type Error = crate::scene::LoaderError;

    fn try_from(data: gltf::image::Data) -> Result<Self, Self::Error> {
        use crate::scene::LoaderError;
        use gltf::image::Format;

        let format = match data.format {
            Format::R8 => gl::RED,
            Format::R8G8 => gl::RG,
            Format::R8G8B8 => gl::RGB,
            Format::R8G8B8A8 => gl::RGBA,
            Format::B8G8R8 => gl::BGR,
            Format::B8G8R8A8 => gl::BGRA,
            _ => {
                return Err(LoaderError::FileError(
                    "Unsuported texture format".to_owned(),
                ))
            }
        };

        unsafe {
            Ok(Self::from_bytes(
                &data.pixels,
                data.width as i32,
                data.height as i32,
                format,
            ))
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
