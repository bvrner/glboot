use std::{fmt::Debug, path::Path};

use crate::ogl::{
    buffers::{array::VertexArray, index::IndexBuffer, vertex::VertexBuffer},
    program::ShaderProgram,
    texture::Texture,
};

use super::VertexData;

use cgmath::Vector4;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub base_color: Vector4<f32>,
    pub base_tex: Option<usize>,

    pub metallic: f32,
    pub roughness: f32,
    pub metallic_tex: Option<usize>,

    pub normal: Option<usize>,
    pub occlusion_tex: Option<usize>,
    pub occlusion_str: f32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            base_tex: None,
            metallic: 1.0,
            roughness: 0.0,
            metallic_tex: None,
            normal: None,
            occlusion_tex: None,
            occlusion_str: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct Mesh<V: VertexData> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
    pub material: Option<usize>,
    pub default_transform: cgmath::Matrix4<f32>,
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
    ) -> Self {
        Mesh {
            vertices,
            indices,
            material,
            default_transform,
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

    fn draw(&self, shader: &mut ShaderProgram, materials: &[Material]) {
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
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            )
        };

        self.ibo.unbind();
        self.vao.unbind();
    }
}

#[derive(Debug)]
pub struct Model<V: VertexData> {
    pub name: String,
    pub meshs: Vec<Mesh<V>>,
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
}

impl<V: VertexData + Send> Model<V> {
    pub fn load<P>(path: P) -> Result<Self, String>
    where
        P: AsRef<Path> + Debug,
    {
        match path.as_ref().extension() {
            Some(ext) if ext == "obj" => super::loaders::load_obj(path),
            Some(ext) if ext == "gltf" => super::loaders::load_gltf(path),
            Some(ext) if ext == "glb" => super::loaders::load_gltf(path),
            _ => Err(String::from("Unsuported file format")),
        }
    }

    pub fn draw(&self, shader: &mut ShaderProgram) {
        shader.bind();

        for (i, tex) in self.textures.iter().enumerate() {
            tex.bind(i as u32);
        }

        for mesh in self.meshs.iter() {
            mesh.draw(shader, &self.materials);
        }
        shader.unbind();
    }
}
