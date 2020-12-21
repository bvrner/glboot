use gl::types::*;

use cgmath::{Vector2, Vector3, Vector4};

use crate::{
    aabb::Aabb,
    ogl::buffers::{array::VertexArray, index::IndexBuffer, vertex::VertexBuffer},
};

#[derive(Debug)]
pub struct Mesh {
    pub primitives: Vec<Primitive>,
    pub name: Option<String>,
    pub aabb: Aabb,
}

#[derive(Debug)]
pub struct Primitive {
    // vertices: Vec<Vertice>, // should I keep them here?
    // indices: Vec<u32>,
    pub material: Option<usize>,
    pub vbo: VertexBuffer,
    pub vao: VertexArray,
    pub indices_count: i32,
    pub vertice_count: i32,
    pub ibo: IndexBuffer,
    aabb: Aabb,
    pub mode: GLenum,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertice {
    pub pos: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex: Vector2<f32>,
    pub joints: Vector4<f32>,
    pub weights: Vector4<f32>,
    pub tangent: Vector4<f32>,
}

impl Mesh {
    pub fn new(prim: Vec<Primitive>, name: Option<String>) -> Self {
        let aabb = prim
            .iter()
            .fold(Aabb::default(), |bound, prim| bound.surrounds(&prim.aabb));
        // dbg!(&name);
        Self {
            primitives: prim,
            aabb,
            name,
        }
    }
}

impl Primitive {
    pub fn setup(
        vertices: Vec<Vertice>,
        indices: Vec<u32>,
        material: Option<usize>,
        aabb: Aabb,
        mode: GLenum,
    ) -> Self {
        let vertice_count = vertices.len() as i32;
        let indices_count = indices.len() as i32;
        let vbo = VertexBuffer::new(&vertices);
        let ibo = IndexBuffer::new(&indices);
        let vao = VertexArray::new();

        let layout = layout![
            (3, f32, gl::FLOAT),
            (3, f32, gl::FLOAT),
            (2, f32, gl::FLOAT),
            (4, f32, gl::FLOAT),
            (4, f32, gl::FLOAT),
            (4, f32, gl::FLOAT)
        ];

        vao.add_buffer(&vbo, &layout);

        Self {
            vbo,
            ibo,
            vao,
            vertice_count,
            indices_count,
            material,
            aabb,
            mode,
        }
    }
}

impl Default for Vertice {
    fn default() -> Self {
        Self {
            pos: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            tex: Vector2::new(0.0, 0.0),
            joints: Vector4::new(0.0, 0.0, 0.0, 0.0),
            weights: Vector4::new(0.0, 0.0, 0.0, 0.0),
            tangent: Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}
