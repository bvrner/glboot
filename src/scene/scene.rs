use crate::{
    ogl::{material::Material, program::ShaderProgram, texture::Texture},
    ImRender,
};
use cgmath::{prelude::*, Matrix4, Quaternion, Vector3};
use thiserror::Error;

use super::{Mesh, Node, Primitive, Vertice};

use rayon::prelude::*;
use std::convert::TryFrom;
use std::path::Path;

#[derive(Debug)]
pub struct Scene {
    nodes: Vec<Node>,  // all nodes
    roots: Vec<usize>, // indices of the roots
    textures: Vec<Texture>,
    materials: Vec<Material>,
    pub scale: f32,
    pub rotation: Quaternion<f32>,
}

impl ImRender for Scene {
    fn render(&mut self, ui: &imgui::Ui) {
        if imgui::CollapsingHeader::new(imgui::im_str!("Scene")).build(&ui) {
            if imgui::Slider::new(imgui::im_str!("Scale"), 0.0001..=1.0).build(&ui, &mut self.scale)
            {
                if self.scale < 0.0001 {
                    self.scale = 0.1
                }
            }
        }
    }
}

impl Scene {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, LoaderError> {
        load_gltf(path)
    }

    pub fn render(&self, shader: &mut ShaderProgram) {
        shader.bind();

        for (i, tex) in self.textures.iter().enumerate() {
            tex.bind(i as u32);
        }

        let transform = Matrix4::from(self.rotation) * Matrix4::from_scale(self.scale);
        // start rendering by the roots which will render it's children and so on and so forth
        for &node in self.roots.iter() {
            self.nodes[node].draw(shader, &self.materials, &self.nodes, transform);
        }
        shader.unbind();
    }
}

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("model loader io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("error in model file: {0}")]
    FileError(String),
    #[error("gltf loader error: {0}")]
    GltfError(#[from] gltf::Error),
}

pub fn load_gltf<P>(path: P) -> Result<Scene, LoaderError>
where
    P: AsRef<Path>,
{
    let (document, buffers, images) = gltf::import(path)?;

    assert_eq!(buffers.len(), document.buffers().count());
    assert_eq!(images.len(), document.images().count());

    let textures: Result<Vec<Texture>, LoaderError> =
        images.into_iter().map(Texture::try_from).collect();
    let textures = textures?;

    let materials: Vec<Material> = document
        .materials()
        .into_iter()
        .par_bridge()
        .map(Material::from)
        .collect();

    let nodes = document
        .nodes()
        .map(|node| process_node(&buffers, &node))
        .collect();

    let mut roots = Vec::new();
    for scene in document.scenes() {
        for node in scene.nodes() {
            roots.push(node.index());
        }
    }

    Ok(Scene {
        roots,
        nodes,
        textures,
        materials,
        scale: 1.0,
        rotation: Quaternion::new(0.0, 0.0, 0.0, 1.0),
    })
}

fn process_node(buffers: &[gltf::buffer::Data], node: &gltf::Node) -> Node {
    let mesh = node.mesh().map(|m| process_mesh(&buffers, &m));
    let transform = node.transform().matrix().into();
    let children = node.children().map(|child| child.index()).collect();

    Node::new(mesh, transform, children)
}

fn process_mesh(buffers: &[gltf::buffer::Data], m: &gltf::Mesh) -> Mesh {
    let primitives = m
        .primitives()
        .map(|primitive| {
            let mut positions = Vec::new();

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(pos) = reader.read_positions() {
                positions.extend(pos.map(|p| Vector3::from(p)));
            }

            //TODO? dynamic vertice types according to the data present?
            let mut vertices: Vec<Vertice> = positions
                .into_iter()
                .map(|p| Vertice {
                    pos: Vector3::from(p),
                    ..Vertice::default()
                })
                .collect();

            if let Some(norm) = reader.read_normals() {
                for (i, normal) in norm.enumerate() {
                    vertices[i].normal = normal.into();
                }
            }

            if let Some(tex) = reader.read_tex_coords(0) {
                for (i, coord) in tex.into_f32().enumerate() {
                    vertices[i].tex = coord.into();
                }
            }

            let indices = if let Some(ind) = reader.read_indices() {
                ind.into_u32().collect()
            } else {
                vec![]
            };
            // let bounds = primitive.bounding_box();

            Primitive::setup(vertices, indices, primitive.material().index())
        })
        .collect();

    Mesh::new(primitives)
}
