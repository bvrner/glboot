use gl::types::*;
use std::{
    ffi::{self, CString},
    fs::File,
    io::{self, Read},
    ops::Drop,
    path::Path,
    ptr,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("shader compiler error: {0}")]
    CompilationError(String),
    #[error("error on FFI: {0}")]
    FfiError(#[from] ffi::NulError),
    #[error("shader io error")]
    IoError(#[from] io::Error),
    #[error("shader source error: {0}")]
    SourceError(String),
    #[error("shader error: {0}")]
    Other(String),
}

// impl From<ffi::NulError> for ShaderError {
//     #[inline]
//     fn from(err: ffi::NulError) -> ShaderError {
//         ShaderError::SourceError(err.to_string())
//     }
// }

pub type VertexShader = Shader<{ gl::VERTEX_SHADER }>;
pub type FragShader = Shader<{ gl::FRAGMENT_SHADER }>;
pub type GeoShader = Shader<{ gl::GEOMETRY_SHADER }>;

// once more const generics features hit the parameter will be an enum
#[derive(Debug)]
pub struct Shader<const TYPE: GLenum>
where
    IsLegal<TYPE>: Truth,
{
    pub(crate) id: GLuint,
}

// a small hack to generate a compiler time error when the enum isn't a valid shader type
// again, when const generics gets better I'll be able to use a custom enum and avoid all that
// this could be made a bit better if complex expressions where allowed in const generic contexts
// so TODO consider using nightly to get that
pub struct IsLegal<const T: GLenum> {}

pub trait Truth {}

// TODO Tesselations shaders?
// TODO also allow shader bits so that this abstraction can also bbe used with separate shader stages
impl Truth for IsLegal<{ gl::VERTEX_SHADER }> {}
impl Truth for IsLegal<{ gl::FRAGMENT_SHADER }> {}
impl Truth for IsLegal<{ gl::GEOMETRY_SHADER }> {}

// #[derive(Copy, Clone, PartialEq, Eq)]
// pub enum ShaderKind {
//     Vertex = gl::VERTEX_SHADER as isize,
//     Fragment = gl::FRAGMENT_SHADER as isize,
//     Geometry = gl::GEOMETRY_SHADER as isize,
// }

// TODO remove IO bound functions
impl<const TYPE: GLenum> Shader<TYPE>
where
    IsLegal<TYPE>: Truth,
{
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self, ShaderError> {
        let mut open_file = File::open(file)?;
        Self::from_reader(&mut open_file)
    }

    pub fn from_reader<R: Read>(source: &mut R) -> Result<Self, ShaderError> {
        let mut shadersrc = String::new();

        source.read_to_string(&mut shadersrc)?;
        Self::from_source(&shadersrc)
    }

    pub fn from_source(source: &str) -> Result<Self, ShaderError> {
        unsafe {
            let shader = gl::CreateShader(TYPE);
            let source = CString::new(source)?;

            gl::ShaderSource(shader, 1, &source.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut success = i32::from(gl::FALSE);
            let mut info_log: Vec<u8> = Vec::new();

            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

            if success != gl::TRUE as GLint {
                let mut log_size = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_size);
                info_log.resize(log_size as usize, 0);
                gl::GetShaderInfoLog(
                    shader,
                    log_size,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );

                gl::DeleteShader(shader);
                let info_log = String::from_utf8_unchecked(info_log);
                Err(ShaderError::CompilationError(info_log))
            } else {
                Ok(Shader { id: shader })
            }
        }
    }
}

impl<const TYPE: GLenum> Drop for Shader<TYPE>
where
    IsLegal<TYPE>: Truth,
{
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) };
    }
}
