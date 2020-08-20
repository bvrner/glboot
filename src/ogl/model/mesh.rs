use std::fmt::Debug;

use crate::ogl::{
    buffers::{array::VertexArray, index::IndexBuffer, vertex::VertexBuffer},
    program::ShaderProgram,
};

use super::{Material, VertexData};
use gl::types::*;

use cgmath::Vector3;

#[derive(Debug)]
pub struct Mesh<V: VertexData> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
    pub material: Option<usize>,
    pub default_transform: cgmath::Matrix4<f32>,
    pub bounds: (Vector3<f32>, Vector3<f32>),
    mode: GLenum,
    vbo: VertexBuffer,
    ibo: IndexBuffer,
    vao: VertexArray,
}

impl<V: VertexData> Mesh<V> {
    pub fn new(
        vertices: Vec<V>,
        indices: Vec<u32>,
        material: Option<usize>,
        default_transform: cgmath::Matrix4<f32>,
        bounds: (Vector3<f32>, Vector3<f32>),
        mode: GLenum,
    ) -> Self {
        Mesh {
            vertices,
            indices,
            material,
            default_transform,
            bounds,
            mode,
            vbo: VertexBuffer::default(),
            ibo: IndexBuffer::default(),
            vao: VertexArray::default(),
        }
    }

    pub fn setup(&mut self) {
        self.vbo = VertexBuffer::new(&self.vertices);
        self.ibo = IndexBuffer::new(&self.indices);
        self.vao = VertexArray::new();
        let layout = V::get_layout();

        self.vao.add_buffer(&self.vbo, &layout);
    }

    pub fn draw(&self, shader: &mut ShaderProgram, materials: &[Material]) {
        shader.set_uniform("default_model", self.default_transform);

        if let Some(mat_index) = self.material {
            let material = &materials[mat_index];

            shader.set_uniform("material.base_color", material.base_color);
            shader.set_uniform("material.has_base_color", 1);

            if let Some(base_tex_index) = material.base_tex {
                shader.set_uniform("material.base_tex", base_tex_index as i32);
                shader.set_uniform("material.has_base_tex", 1);
            } else {
                shader.set_uniform("material.has_base_tex", 0);
            }
        }

        self.vao.bind();
        self.ibo.bind();
        shader.send_uniforms();
        unsafe {
            gl::DrawElements(
                self.mode,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            )
        };

        self.ibo.unbind();
        self.vao.unbind();
    }
}
