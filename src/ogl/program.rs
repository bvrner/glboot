use super::shaders::{Shader, ShaderError, ShaderKind};
use std::{collections::HashMap, ffi::CString, fs::File, io::Read, path::Path, ptr};

use gl::types::*;

use cgmath::prelude::*;
use cgmath::{Array, Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};

#[derive(Debug)]
pub struct ShaderProgram(GLuint, HashMap<GLint, Uniform>);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Uniform {
    Float(f32),
    Int(i32),
    Vec3(Vector3<f32>),
    Vec4(Vector4<f32>),
    Vec2(Vector2<f32>),
    Mat2(Matrix2<f32>),
    Mat3(Matrix3<f32>),
    Mat4(Matrix4<f32>),
}

impl ShaderProgram {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ShaderError> {
        let src = {
            let mut file = File::open(path)?;
            let mut src = String::new();

            file.read_to_string(&mut src)?;
            src
        };

        let (v, f, g) = process_all(src)?;
        Self::from_shaders(v, f, g)
    }

    pub fn from_files<P: AsRef<Path>>(
        v_path: P,
        f_path: P,
        _g_path: Option<P>,
    ) -> Result<Self, ShaderError> {
        let (v, f) = (
            Shader::from_file(v_path, ShaderKind::Vertex)?,
            Shader::from_file(f_path, ShaderKind::Fragment)?,
        );

        Self::from_shaders(v, f, None)
    }

    pub fn from_readers<R: Read>(
        v_reader: &mut R,
        f_reader: &mut R,
        _g_reader: R,
    ) -> Result<Self, ShaderError> {
        let (v, f) = (
            Shader::from_reader(v_reader, ShaderKind::Vertex)?,
            Shader::from_reader(f_reader, ShaderKind::Fragment)?,
        );

        Self::from_shaders(v, f, None)
    }

    pub fn from_shaders(
        vertex: Shader,
        frag: Shader,
        geo: Option<Shader>,
    ) -> Result<Self, ShaderError> {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex.0);
            gl::AttachShader(program, frag.0);
            if let Some(ref geo) = geo {
                gl::AttachShader(program, geo.0);
            }

            gl::LinkProgram(program);

            check_program_status(program, gl::LINK_STATUS)?;

            gl::ValidateProgram(program);

            check_program_status(program, gl::VALIDATE_STATUS)?;

            gl::DetachShader(program, vertex.0);
            gl::DetachShader(program, frag.0);

            if let Some(ref geo) = geo {
                gl::DetachShader(program, geo.0);
            }
            Ok(ShaderProgram(program, HashMap::new()))
        }
    }

    #[inline]
    pub fn bind(&self) {
        unsafe { gl::UseProgram(self.0) };
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe { gl::UseProgram(0) };
    }

    pub fn set_uniform<T: Into<Uniform>>(&mut self, name: &str, uniform: T) {
        let ffi_name = CString::new(name).unwrap();

        let location = unsafe { gl::GetUniformLocation(self.0, ffi_name.as_ptr()) };

        if location != -1 {
            self.1.insert(location, uniform.into());
        }
    }

    pub fn send_uniforms(&self) {
        for (location, uniform) in self.1.iter() {
            match uniform {
                Uniform::Float(f) => unsafe { gl::Uniform1f(*location, *f) },
                Uniform::Int(i) => unsafe { gl::Uniform1i(*location, *i) },
                Uniform::Vec2(v) => unsafe { gl::Uniform2fv(*location, 1, v.as_ptr()) },
                Uniform::Vec3(v) => unsafe { gl::Uniform3fv(*location, 1, v.as_ptr()) },
                Uniform::Vec4(v) => unsafe { gl::Uniform4fv(*location, 1, v.as_ptr()) },
                Uniform::Mat2(m) => unsafe {
                    gl::UniformMatrix2fv(*location, 1, gl::FALSE, m.as_ptr())
                },
                Uniform::Mat3(m) => unsafe {
                    gl::UniformMatrix3fv(*location, 1, gl::FALSE, m.as_ptr())
                },
                Uniform::Mat4(m) => unsafe {
                    gl::UniformMatrix4fv(*location, 1, gl::FALSE, m.as_ptr())
                },
            }
        }
    }
}

fn check_program_status(program: GLuint, which: GLenum) -> Result<(), ShaderError> {
    unsafe {
        let mut status = gl::TRUE as i32;

        gl::GetProgramiv(program, which, &mut status);

        if status == gl::TRUE as i32 {
            Ok(())
        } else {
            let mut length = 0;
            let mut info_log: Vec<u8> = Vec::new();

            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length);
            info_log.resize(length as usize, 0);

            gl::GetProgramInfoLog(
                program,
                length,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );

            let info_log = String::from_utf8_unchecked(info_log);
            gl::DeleteProgram(program);
            Err(ShaderError::Other(info_log))
        }
    }
}

// this works but it's kinda slow, TODO optmize and properly deal with errors
// considering using the glsl crate to properly parse the source
fn process_all(src: String) -> Result<(Shader, Shader, Option<Shader>), ShaderError> {
    const V_BEGIN_MARK: &str = "#begin vertex";
    const F_BEGIN_MARK: &str = "#begin fragment";
    const G_BEGIN_MARK: &str = "#begin geometry";

    const V_END_MARK: &str = "#end vertex";
    const F_END_MARK: &str = "#end fragment";
    const G_END_MARK: &str = "#end geometry";

    // small helper closure
    let opt_to_result = |opt: Option<usize>, err| opt.ok_or(ShaderError::SourceError(err));

    let (v_begin, v_end) = (src.find(V_BEGIN_MARK), src.find(V_END_MARK));
    let (f_begin, f_end) = (src.find(F_BEGIN_MARK), src.find(F_END_MARK));
    let geometry = (src.find(G_BEGIN_MARK), src.find(G_END_MARK));

    let v_begin = opt_to_result(v_begin, "No begin point for vertex shader".to_owned())?;
    let v_end = opt_to_result(v_end, "No end point for vertex shader".to_owned())?;
    let f_begin = opt_to_result(f_begin, "No begin point for fragment shader".to_owned())?;
    let f_end = opt_to_result(f_end, "No end point for fragment shader".to_owned())?;

    let v_shader = Shader::from_source(
        &src[(v_begin + V_BEGIN_MARK.len())..v_end],
        ShaderKind::Vertex,
    )?;
    let f_shader = Shader::from_source(
        &src[(f_begin + F_BEGIN_MARK.len())..f_end],
        ShaderKind::Fragment,
    )?;

    // oh god is that ugly
    // I could use Option's `zip` to make it more readable but I don't want to go to nightly
    let g_shader = match geometry {
        (Some(begin), Some(end)) => Some(Shader::from_source(
            &src[(begin + G_BEGIN_MARK.len()..end)],
            ShaderKind::Geometry,
        )?),
        (None, None) => None,
        (None, _) => {
            return Err(ShaderError::SourceError(
                "No begin point for geometry shader".to_owned(),
            ))
        }
        (_, None) => {
            return Err(ShaderError::SourceError(
                "No end point for geometry shader".to_owned(),
            ))
        }
    };

    Ok((v_shader, f_shader, g_shader))
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.0) };
    }
}

// just so I don't end up filling 50 lines of that
macro_rules! impl_into_uni {
    ($which:ty, $to:expr) => {
        impl Into<Uniform> for $which {
            #[inline]
            fn into(self) -> Uniform {
                $to(self)
            }
        }
    };
}

impl_into_uni!(f32, Uniform::Float);
impl_into_uni!(i32, Uniform::Int);
impl_into_uni!(Vector2<f32>, Uniform::Vec2);
impl_into_uni!(Vector3<f32>, Uniform::Vec3);
impl_into_uni!(Vector4<f32>, Uniform::Vec4);
impl_into_uni!(Matrix2<f32>, Uniform::Mat2);
impl_into_uni!(Matrix3<f32>, Uniform::Mat3);
impl_into_uni!(Matrix4<f32>, Uniform::Mat4);
