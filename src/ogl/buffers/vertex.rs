use gl::types::*;
use std::{ffi::c_void, mem};

#[derive(Debug, Default)]
pub struct VertexBuffer(GLuint);

impl VertexBuffer {
    pub fn new<T>(data: &[T]) -> Self {
        unsafe {
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferStorage(
                gl::ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
                0,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            VertexBuffer(vbo)
        }
    }

    #[inline]
    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.0) };
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0) };
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.0) };
    }
}
