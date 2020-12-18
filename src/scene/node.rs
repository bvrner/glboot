use crate::{
    aabb::Aabb,
    ogl::{material::Material, program::ShaderProgram},
};

use super::Mesh;
use cgmath::{Matrix, Matrix4, Quaternion, SquareMatrix, Vector3};

#[derive(Debug)]
pub struct Node {
    pub mesh: Option<Mesh>,

    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,

    pub global_transform: Matrix4<f32>,
    pub transform: Matrix4<f32>, // cached local transformation

    pub skin: Option<usize>,
    pub children: Vec<usize>, // the indices of this node children, see the Scene struct
                              //camera: Camera ???
}

impl Node {
    pub fn new(
        mesh: Option<Mesh>,
        transform: gltf::scene::Transform,
        children: Vec<usize>,
        skin: Option<usize>,
    ) -> Self {
        let (translation, rotation, scale) = transform.clone().decomposed();
        let transform = transform.matrix().into();

        Self {
            mesh,
            transform,
            children,
            skin,
            global_transform: transform,
            translation: translation.into(),
            rotation: Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2]),
            scale: scale.into(),
        }
    }

    pub fn update_global(&mut self, parent: Matrix4<f32>) {
        self.global_transform = parent * self.transform;
    }

    pub fn update(&mut self) {
        let transform = Matrix4::from_translation(self.translation)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        self.transform = transform;
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
        skins: &[super::skin::Skin], // transform: Matrix4<f32>,
    ) {
        if let Some(ref mesh) = self.mesh {
            if let Some(skin) = self.skin {
                let joints = &skins[skin].joints;
                let joint_matrices: Vec<Matrix4<f32>> = joints
                    .iter()
                    .map(|joint| {
                        self.global_transform.invert().unwrap()
                            * nodes[joint.node].global_transform
                            * joint.bind_matrix
                    })
                    .collect();

                // dbg!(joint_matrices.len());

                unsafe {
                    let name = std::ffi::CString::new("joints").unwrap();
                    gl::UniformMatrix4fv(
                        gl::GetUniformLocation(shader.0, name.as_ptr()),
                        joint_matrices.len() as i32,
                        gl::FALSE,
                        joint_matrices[0].as_ptr(),
                    );
                }
            }
            mesh.draw(shader, materials, self.global_transform);
        }

        for &child in self.children.iter() {
            nodes[child].draw(
                shader, materials, nodes, skins, /*, self.global_transform*/
            );
        }
    }
}

pub fn build_tree(nodes: &[Node], roots: &[usize]) -> Vec<(usize, Option<usize>)> {
    let mut ret = Vec::with_capacity(nodes.len());

    for &root in roots.iter() {
        build_tree_helper(nodes, root, None, &mut ret);
    }

    ret.shrink_to_fit();
    ret
}

fn build_tree_helper(
    nodes: &[Node],
    current: usize,
    parent: Option<usize>,
    indices: &mut Vec<(usize, Option<usize>)>,
) {
    indices.push((current, parent));

    for &child in nodes[current].children.iter() {
        build_tree_helper(nodes, child, Some(current), indices);
    }
}
