use gl::types::*;
use thiserror::Error;

// quick dirty macro to help create the builders
macro_rules! gen_tex_builder {
    // generate the struct and setters
    ($name:ident  { $( ($field:ident, $t:ty ) ),* } ) => {

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name {

            $( $field: $t , )+
        }

        impl $name {
            $(
                pub fn $field(&mut self, arg: $t) -> &mut Self {
                    self.$field = arg;
                    self
                }
            )+

        }
    }
}

// TODO use Rust enums instead of plain OpenGL values for texture options
mod tex1d;
mod tex2d;
mod tex3d;

pub use tex1d::{Texture1D, TextureBuilder1D};
pub use tex2d::{Texture2D, TextureBuilder2D};
pub use tex3d::{Texture3D, TextureBuilder3D};

// currently unused
#[derive(Debug, Error)]
pub enum TextureError {
    #[error("image loader failed with: ")]
    Loading(#[from] image::ImageError),
    #[error("invalid data passed to builder: {0}")]
    InvalidData(String),
}
