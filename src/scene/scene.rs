use crate::ogl::{material::Material, program::ShaderProgram, texture::Texture};
use cgmath::{prelude::*, Matrix4, Vector2, Vector3};
use thiserror::Error;

use super::{Mesh, Node, Primitive, Vertice};

use rayon::prelude::*;
use std::path::Path;

#[derive(Debug)]
pub struct Scene {
    roots: Vec<Node>,
    textures: Vec<Texture>,
    materials: Vec<Material>,
    // transform: Matrix4<f32>
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

        for node in self.roots.iter() {
            node.draw(shader, &self.materials, Matrix4::from_scale(0.005));
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

    let textures: Result<Vec<Texture>, LoaderError> = images
        .into_iter()
        .map(|data| {
            use gltf::image::Format;

            let format = match data.format {
                Format::R8 => gl::RED,
                Format::R8G8 => gl::RG,
                Format::R8G8B8 => gl::RGB,
                Format::R8G8B8A8 => gl::RGBA,
                Format::B8G8R8 => gl::BGR,
                Format::B8G8R8A8 => gl::BGRA,
                _ => {
                    return Err(LoaderError::FileError(
                        "Unsuported texture format".to_owned(),
                    ))
                }
            };

            unsafe {
                Ok(Texture::from_bytes(
                    &data.pixels,
                    data.width as i32,
                    data.height as i32,
                    format,
                ))
            }
        })
        .collect();
    let textures = textures?;

    let materials: Vec<Material> = document
        .materials()
        .into_iter()
        .par_bridge()
        .map(|mat| {
            let metallic_roughness = mat.pbr_metallic_roughness();
            let base_color = metallic_roughness.base_color_factor().into();
            let base_tex = metallic_roughness
                .base_color_texture()
                .map(|info| info.texture().index());

            let metallic = metallic_roughness.metallic_factor();
            let roughness = metallic_roughness.roughness_factor();
            let metallic_tex = metallic_roughness
                .metallic_roughness_texture()
                .map(|info| info.texture().index());

            let normal = mat.normal_texture().map(|norm| norm.texture().index());
            let (occlusion_tex, occlusion_str) = mat
                .occlusion_texture()
                .map(|occ| (Some(occ.texture().index()), occ.strength()))
                .unwrap_or((None, 0.0));

            Material {
                base_color,
                base_tex,
                metallic,
                roughness,
                metallic_tex,
                normal,
                occlusion_tex,
                occlusion_str,
            }
        })
        .collect();

    let mut roots = Vec::new();
    for scene in document.scenes() {
        for node in scene.nodes() {
            roots.push(proccess_node(&buffers, &node));
        }
    }

    Ok(Scene {
        roots,
        textures,
        materials,
    })
}

fn proccess_node(buffers: &[gltf::buffer::Data], node: &gltf::Node) -> Node {
    let mesh = node.mesh().map(|m| proccess_mesh(&buffers, &m));
    let transform = node.transform().matrix().into();
    let children = node
        .children()
        .map(|child| proccess_node(&buffers, &child))
        .collect();

    Node::new(mesh, transform, children)
}

fn proccess_mesh(buffers: &[gltf::buffer::Data], m: &gltf::Mesh) -> Mesh {
    let primitives = m
        .primitives()
        .map(|primitive| {
            let mut indices: Vec<u32> = Vec::new();
            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut tex_coords = Vec::new();

            let material = primitive.material().index();

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(pos) = reader.read_positions() {
                positions.extend(pos.map(|p| Vector3::from(p)));
            }

            if let Some(norm) = reader.read_normals() {
                normals.extend(norm.map(|n| Vector3::from(n)));
            }

            if let Some(tex) = reader.read_tex_coords(0) {
                tex_coords.extend(tex.into_f32().map(|t| Vector2::from(t)));
            }

            if let Some(ind) = reader.read_indices() {
                indices.extend(ind.into_u32());
            }

            let normal_iter = if normals.len() > 0 {
                normals.into_iter().cycle()
            } else {
                vec![Vector3::new(0.0_f32, 0.0, 0.0)].into_iter().cycle()
            };
            let tex_iter = if tex_coords.len() > 0 {
                tex_coords.into_iter().cycle()
            } else {
                vec![Vector2::new(0.0_f32, 0.0)].into_iter().cycle()
            };

            let vertices = positions
                .into_iter()
                .zip(normal_iter)
                .zip(tex_iter)
                .map(|((pos, normal), tex)| Vertice { pos, normal, tex })
                .collect();

            Primitive::setup(vertices, indices, material)

            // let bounds = primitive.bounding_box();
        })
        .collect();

    Mesh::new(primitives)
}
