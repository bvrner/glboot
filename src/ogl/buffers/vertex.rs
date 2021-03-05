use gl::types::*;
use std::{ffi::c_void, mem};

// TODO consided parametrizing this type with a generic type
#[derive(Debug, Default)]
pub struct VertexBuffer(GLuint);

impl VertexBuffer {
    /// Creates a new vertex buffer initialized with the slice data.
    pub fn new<T>(data: &[T]) -> Self {
        unsafe {
            let mut vbo = 0;
            gl::CreateBuffers(1, &mut vbo);

            gl::NamedBufferStorage(
                vbo,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
                0,
            );

            VertexBuffer(vbo)
        }
    }

    /// Creates a new empty buffer with `size` * `size_of::<T>()` bytes
    pub fn empty<T>(size: usize) -> Self {
        unsafe {
            let mut vbo = 0;

            gl::CreateBuffers(1, &mut vbo);

            gl::NamedBufferStorage(
                vbo,
                (size * mem::size_of::<T>()) as GLsizeiptr,
                std::ptr::null(),
                0,
            );

            VertexBuffer(vbo)
        }
    }

    pub fn write<T>(&self, offset: GLintptr, data: &[T]) {
        unsafe {
            gl::NamedBufferSubData(
                self.0,
                offset,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
            );
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
