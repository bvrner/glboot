use crate::ogl::material::Material;
use crate::ogl::program::ShaderProgram;

use super::Mesh;
use cgmath::Matrix4;

#[derive(Debug)]
pub struct Node {
    mesh: Option<Mesh>,
    transform: Matrix4<f32>,
    children: Vec<Node>, //camera: Camera ???
}

impl Node {
    pub fn new(mesh: Option<Mesh>, transform: Matrix4<f32>, children: Vec<Node>) -> Self {
        Self {
            mesh,
            transform,
            children,
        }
    }

    pub fn draw(
        &self,
        shader: &mut ShaderProgram,
        materials: &[Material],
        transform: Matrix4<f32>,
    ) {
        if let Some(ref mesh) = self.mesh {
            mesh.draw(shader, materials, transform * self.transform);
        }

        for child in self.children.iter() {
            child.draw(shader, materials, transform * self.transform);
        }
    }
}
