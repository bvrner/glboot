use gl::types::*;
use std::{
    error::Error,
    ffi::{self, CString},
    fmt,
    fs::File,
    io::{self, Read},
    ops::Drop,
    path::Path,
    ptr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShaderError {
    CompilationError(String),
    SourceError(String),
    IoError(String),
    Other(String),
}

impl Error for ShaderError {}
impl fmt::Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShaderError::CompilationError(s) => write!(f, "Shader compilation error: {}", s),
            ShaderError::SourceError(s) => write!(f, "Shader source error: {}", s),
            ShaderError::IoError(s) => write!(f, "Shader IO error: {}", s),
            ShaderError::Other(s) => write!(f, "Shader error: {}", s),
        }
    }
}
impl From<io::Error> for ShaderError {
    #[inline]
    fn from(err: io::Error) -> ShaderError {
        ShaderError::IoError(err.to_string())
    }
}

impl From<ffi::NulError> for ShaderError {
    #[inline]
    fn from(err: ffi::NulError) -> ShaderError {
        ShaderError::SourceError(err.to_string())
    }
}

#[derive(Debug)]
pub struct Shader(pub(crate) GLuint);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ShaderKind {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize,
}

impl Shader {
    pub fn from_file<P: AsRef<Path>>(file: P, kind: ShaderKind) -> Result<Self, ShaderError> {
        let mut open_file = File::open(file)?;
        Self::from_reader(&mut open_file, kind)
    }

    pub fn from_reader<R: Read>(source: &mut R, kind: ShaderKind) -> Result<Self, ShaderError> {
        let mut shadersrc = String::new();

        source.read_to_string(&mut shadersrc)?;
        Self::from_source(&shadersrc, kind)
    }

    pub fn from_source(source: &str, kind: ShaderKind) -> Result<Self, ShaderError> {
        unsafe {
            let shader = gl::CreateShader(kind as GLenum);
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
                Ok(Shader(shader))
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.0) };
    }
}
