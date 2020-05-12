use gl::types::*;
use std::{convert::TryInto, ffi::c_void, mem};

use super::vertex::VertexBuffer;

#[derive(Debug, Default)]
pub struct BufferElement {
    pub count: i32,
    pub ty: GLenum,
    pub size: u32,
}

#[derive(Debug, Default)]
pub struct Layout {
    elements: Vec<BufferElement>,
    stride: i32,
}

impl Layout {
    pub fn push<T>(&mut self, count: i32, ty: GLenum) {
        let size = mem::size_of::<T>();

        self.elements.push(BufferElement {
            count,
            ty,
            size: size as u32,
        });

        self.stride += size as i32 * count;
    }
}

// Small little helper macro to create layouts, similar to the vec! macro in the standard library.
#[macro_export]
macro_rules! layout {
    ($( ($amount:expr, $t:ty, $kind:expr) ),*) => {
        {
            let mut l = $crate::Layout::default();

            $(
                l.push::<$t>($amount, $kind);
            )*

            l
        }
    }
}

#[derive(Debug)]
pub struct VertexArray(GLuint);

impl VertexArray {
    pub fn new() -> Self {
        unsafe {
            let mut id = 0;
            gl::GenVertexArrays(1, &mut id);
            VertexArray(id)
        }
    }

    #[inline]
    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.0) };
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe { gl::BindVertexArray(0) };
    }

    pub fn add_buffer(&self, buffer: &VertexBuffer, layout: &Layout) {
        self.bind();
        buffer.bind();

        let mut offset = 0;
        for (i, elem) in layout.elements.iter().enumerate() {
            unsafe {
                gl::EnableVertexAttribArray(i.try_into().unwrap());
                gl::VertexAttribPointer(
                    i.try_into().unwrap(),
                    elem.count,
                    elem.ty,
                    gl::FALSE,
                    layout.stride,
                    (offset * elem.size) as *const c_void,
                );
            }
            offset += elem.count as u32;
        }

        self.unbind();
        buffer.unbind();
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &self.0) };
    }
}
