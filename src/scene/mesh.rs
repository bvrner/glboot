use cgmath::Matrix4;
use cgmath::{Vector2, Vector3};

use crate::{
    ogl::{
        buffers::{array::VertexArray, index::IndexBuffer, vertex::VertexBuffer},
        material::Material,
    },
    ShaderProgram,
};

#[derive(Debug)]
pub struct Mesh {
    primitives: Vec<Primitive>,
    // aabb: Aabb
}

#[derive(Debug)]
pub struct Primitive {
    // vertices: Vec<Vertice>, // should I keep them here?
    // indices: Vec<u32>,
    material: Option<usize>,
    vbo: VertexBuffer,
    vao: VertexArray,
    indices_count: i32,
    ibo: IndexBuffer, //aabb: Aabb
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertice {
    pub pos: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex: Vector2<f32>,
}

impl Mesh {
    pub fn new(prim: Vec<Primitive>) -> Self {
        Self { primitives: prim }
    }

    pub fn draw(
        &self,
        shader: &mut ShaderProgram,
        materials: &[Material],
        transform: Matrix4<f32>,
    ) {
        for prim in self.primitives.iter() {
            prim.draw(shader, materials, transform);
        }
    }
}

impl Primitive {
    pub fn draw(
        &self,
        shader: &mut ShaderProgram,
        materials: &[Material],
        transform: Matrix4<f32>,
    ) {
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

        shader.set_uniform("model", transform);

        self.vao.bind();
        self.ibo.bind();
        shader.send_uniforms();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            )
        };

        self.ibo.unbind();
        self.vao.unbind();
    }

    pub fn setup(vertices: Vec<Vertice>, indices: Vec<u32>, material: Option<usize>) -> Self {
        let indices_count = indices.len() as i32;
        let vbo = VertexBuffer::new(&vertices);
        let ibo = IndexBuffer::new(&indices);
        let vao = VertexArray::new();

        let layout = layout![
            (3, f32, gl::FLOAT),
            (3, f32, gl::FLOAT),
            (2, f32, gl::FLOAT)
        ];

        vao.add_buffer(&vbo, &layout);

        Self {
            vbo,
            ibo,
            vao,
            indices_count,
            material,
        }
    }
}
