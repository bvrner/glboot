use crate::ogl::material::Material;
use crate::ogl::program::ShaderProgram;

use super::Mesh;
use cgmath::Matrix4;

#[derive(Debug)]
pub struct Node {
    pub mesh: Option<Mesh>,
    transform: Matrix4<f32>,
    children: Vec<usize>, // the indices of this node children, see the Scene struct
                          //camera: Camera ???
}

impl Node {
    pub fn new(mesh: Option<Mesh>, transform: Matrix4<f32>, children: Vec<usize>) -> Self {
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
        nodes: &[Node],
        transform: Matrix4<f32>,
    ) {
        if let Some(ref mesh) = self.mesh {
            mesh.draw(shader, materials, transform * self.transform);
        }

        for &child in self.children.iter() {
            nodes[child].draw(shader, materials, nodes, transform * self.transform);
        }
    }
}
