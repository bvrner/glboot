use std::{fmt::Debug, path::Path, sync::Arc};

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
    pub tangent: Vector3<f32>,
    pub bitangent: Vector3<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
    pub shininess: f32,
}

#[derive(Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub textures: Option<Arc<Vec<(String, Texture)>>>,
    pub indices: Vec<u32>,
    pub material: Option<Material>,
    vbo: VertexBuffer,
    ibo: IndexBuffer,
    vao: VertexArray,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        textures: Option<Arc<Vec<(String, Texture)>>>, // god I love generics syntax
        indices: Vec<u32>,
        material: Option<Material>,
    ) -> Self {
        Mesh {
            vertices,
            textures,
            indices,
            material,
            ..Default::default()
        }
    }

    pub fn setup(&mut self) {
        // Compute the tangent and bitanget for each triangle on the mesh
        // I'll just trust no indice goes out of bounds
        for triangle in self.indices.chunks_exact_mut(3) {
            let v0 = &self.vertices[triangle[0] as usize].vertice;
            let v1 = &self.vertices[triangle[1] as usize].vertice;
            let v2 = &self.vertices[triangle[2] as usize].vertice;

            let uv0 = &self.vertices[triangle[0] as usize].tex_coords;
            let uv1 = &self.vertices[triangle[1] as usize].tex_coords;
            let uv2 = &self.vertices[triangle[2] as usize].tex_coords;

            let delta1 = v1 - v0;
            let delta2 = v2 - v0;

            let deltau1 = uv1 - uv0;
            let deltau2 = uv2 - uv0;

            let r = 1.0 / (deltau1.x * deltau2.y - deltau1.y * deltau2.x);

            let tangent = (delta1 * deltau2.y - delta2 * deltau1.y) * r;
            let bitangent = (delta2 * deltau1.x - delta1 * deltau2.x) * r;

            self.vertices[triangle[0] as usize].tangent = tangent;
            self.vertices[triangle[1] as usize].tangent = tangent;
            self.vertices[triangle[2] as usize].tangent = tangent;

            self.vertices[triangle[0] as usize].bitangent = bitangent;
            self.vertices[triangle[1] as usize].bitangent = bitangent;
            self.vertices[triangle[2] as usize].bitangent = bitangent;
        }

        self.vbo = VertexBuffer::new(&self.vertices);
        self.ibo = IndexBuffer::new(&self.indices);
        self.vao = VertexArray::new();
        let layout = layout![
            (3, f32, gl::FLOAT),
            (3, f32, gl::FLOAT),
            (2, f32, gl::FLOAT),
            (3, f32, gl::FLOAT),
            (3, f32, gl::FLOAT)
        ];

        self.vao.add_buffer(&self.vbo, &layout);
    }

    pub fn draw(&self, shader: &mut ShaderProgram) {
        if let Some(ref textures) = self.textures {
            for (index, (name, tex)) in textures.iter().enumerate() {
                shader.set_uniform(&name, index as i32);
                tex.bind(index as u32);
            }
        }

        // TODO send multiple uniforms with one call
        // OR use uniform buffers instead
        if let Some(ref material) = self.material {
            shader.set_uniform("material.has_specular", 1);
            shader.set_uniform("material.has_diffuse", 1);

            shader.set_uniform("material.ambient", material.ambient);
            shader.set_uniform("material.shininess", material.shininess);
            shader.set_uniform("material.specular", material.specular);
            shader.set_uniform("material.diffuse", material.diffuse);
        } else {
            shader.set_uniform("material.has_specular", 0);
            shader.set_uniform("material.has_diffuse", 0);
        }

        shader.bind();
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
        super::loaders::load_obj(path)
    }

    pub fn draw(&self, shader: &mut ShaderProgram) {
        for mesh in self.meshs.iter() {
            mesh.draw(shader);
        }
    }
}
