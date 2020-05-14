use std::{fmt::Debug, path::Path};

use crate::ogl::{
    buffers::{array::VertexArray, index::IndexBuffer, vertex::VertexBuffer},
    program::ShaderProgram,
    texture::Texture,
};

use cgmath::{Vector2, Vector3};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub vertice: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coords: Vector2<f32>,
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub textures: Vec<Texture>,
    pub indices: Vec<u32>,
    vbo: VertexBuffer,
    ibo: IndexBuffer,
    vao: VertexArray,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, textures: Vec<Texture>, indices: Vec<u32>) -> Self {
        let vbo = VertexBuffer::new(&vertices);
        let ibo = IndexBuffer::new(&indices);
        let vao = VertexArray::new();
        let layout = layout![
            (3, f32, gl::FLOAT),
            (3, f32, gl::FLOAT),
            (2, f32, gl::FLOAT)
        ];

        vao.add_buffer(&vbo, &layout);

        Mesh {
            vertices,
            textures,
            indices,
            vbo,
            ibo,
            vao,
        }
    }

    pub fn draw(&self, shader: &ShaderProgram) {
        shader.bind();
        shader.send_uniforms();
        self.vao.bind();
        self.ibo.bind();

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            )
        };

        shader.unbind();
        self.ibo.unbind();
        self.vao.unbind();
    }
}

#[derive(Debug)]
pub struct Model {
    pub name: String,
    pub meshs: Vec<Mesh>,
}

impl Model {
    pub fn load<P>(path: P) -> Result<Self, String>
    where
        P: AsRef<Path> + Debug,
    {
        super::obj::load_obj(path)
    }

    pub fn draw(&self, shader: &ShaderProgram) {
        for mesh in self.meshs.iter() {
            mesh.draw(shader);
        }
    }
}
