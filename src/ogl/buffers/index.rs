use gl::types::*;
use std::{ffi::c_void, mem};

#[derive(Debug)]
pub struct IndexBuffer(GLuint);

impl IndexBuffer {
    pub fn new<T>(data: &[T]) -> Self {
        unsafe {
            let mut buffer: GLuint = 0;

            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            IndexBuffer(buffer)
        }
    }

    #[inline]
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.0);
        }
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.0);
        }
    }
}
