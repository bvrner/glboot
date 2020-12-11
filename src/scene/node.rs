use crate::{
    aabb::Aabb,
    ogl::{material::Material, program::ShaderProgram},
};

use super::Mesh;
use cgmath::{Matrix4, Quaternion, Vector3};

#[derive(Debug)]
pub struct Node {
    pub mesh: Option<Mesh>,

    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,

    pub transform: Matrix4<f32>, // cached transformation

    pub children: Vec<usize>, // the indices of this node children, see the Scene struct
                              //camera: Camera ???
}

impl Node {
    pub fn new(
        mesh: Option<Mesh>,
        transform: gltf::scene::Transform,
        children: Vec<usize>,
    ) -> Self {
        let (translation, rotation, scale) = transform.clone().decomposed();
        let transform = transform.matrix().into();

        Self {
            mesh,
            transform,
            children,
            translation: translation.into(),
            rotation: Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2]),
            scale: scale.into(),
        }
    }

    pub fn gen_aabb(&self, nodes: &[Node], transform: Matrix4<f32>) -> Aabb {
        let mut this_aabb = if let Some(ref mesh) = self.mesh {
            mesh.aabb.transform(&transform)
        } else {
            Aabb::default()
        };

        for &child in self.children.iter() {
            let child_aabb = nodes[child].gen_aabb(nodes, transform * self.transform);

            this_aabb = this_aabb.surrounds(&child_aabb);
        }

        this_aabb
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
