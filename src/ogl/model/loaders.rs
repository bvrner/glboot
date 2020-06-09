use super::mesh::{Material, Mesh, Model};
use crate::ogl::texture::Texture;
use cgmath::{vec2, vec3, Vector2, Vector3};

use super::{RawVertex, VertexData};
use rayon::prelude::*;

use std::{
    // error::Error,
    fmt::Debug,
    path::Path,
    sync::Arc,
};

// TODO General error type for all handlers
#[derive(Debug, Clone)]
pub enum LoaderError {
    IOError,
    FileError,
}

pub fn load_obj<P, V>(path: P) -> Result<Model<V>, String>
where
    P: AsRef<Path> + Debug,
    V: VertexData + Send,
{
    let (models, materials) = tobj::load_obj(&path, true).unwrap();

    let (materials, textures) = materials.into_iter().fold(
        (vec![], vec![]), // confusing, I know, sorry
        |(mut vec, mut texts), mat| {
            vec.push(Material {
                specular: mat.specular.into(),
                diffuse: mat.diffuse.into(),
                ambient: mat.ambient.into(),
                shininess: mat.shininess,
            });

            let mut text_vec = Vec::with_capacity(3);
            let dir = path.as_ref().parent().unwrap().to_str().unwrap();

            // 1. diffuse map
            if !mat.diffuse_texture.is_empty() {
                text_vec.push((
                    "material.diffuse".to_owned(),
                    Texture::new(&format!("{}/{}", dir, mat.diffuse_texture), false).unwrap(),
                ));
            }
            // 2. specular map
            if !mat.specular_texture.is_empty() {
                text_vec.push((
                    "material.specular".to_owned(),
                    Texture::new(&format!("{}/{}", dir, mat.specular_texture), false).unwrap(),
                ));
            }

            // normal map
            if !mat.normal_texture.is_empty() {
                text_vec.push((
                    "material.normal".to_owned(),
                    Texture::new(&format!("{}/{}", dir, mat.normal_texture), false).unwrap(),
                ));
            }
            texts.push(Arc::new(text_vec));
            (vec, texts)
        },
    );

    let mut meshs: Vec<Mesh<V>> = models
        .into_par_iter()
        .map(|model| {
            let mesh = model.mesh;

            let indices: Vec<u32> = mesh.indices;

            let raw = {
                let (p, n, t) = (mesh.positions, mesh.normals, mesh.texcoords);

                let v: Vec<Vector3<f32>> =
                    p.chunks_exact(3).map(|p| vec3(p[0], p[1], p[2])).collect();
                let n: Vec<Vector3<f32>> =
                    n.chunks_exact(3).map(|p| vec3(p[0], p[1], p[2])).collect();
                let t: Vec<Vector2<f32>> = t.chunks_exact(2).map(|p| vec2(p[0], p[1])).collect();

                RawVertex {
                    vertices: v,
                    normals: n,
                    tex_coords: t,
                }
            };

            let (material, texture) = if let Some(index) = mesh.material_id {
                (Some(materials[index]), Some(textures[index].clone()))
            } else {
                (None, None)
            };

            Mesh::new(V::from_raw(raw), texture, indices, material)
        })
        .collect();

    // OpenGL doesn't like when we do it inside the parallel iterator
    meshs.iter_mut().for_each(|m| m.setup());

    Ok(Model {
        name: String::new(),
        meshs,
    })
}

pub fn load_gltf<P, V>(path: P) -> Result<Model<V>, String>
where
    P: AsRef<Path>,
    V: VertexData,
{
    let (document, buffers, images) = gltf::import(path).unwrap();

    assert_eq!(buffers.len(), document.buffers().count());
    assert_eq!(images.len(), document.images().count());

    let mut meshs = Vec::new();

    for node in document.nodes() {
        let mut stack = vec![node];

        while let Some(node) = stack.pop() {
            if let Some(mesh) = process_node(&node, &buffers, &images) {
                meshs.push(mesh);
            }

            for child in node.children() {
                stack.push(child);
            }
        }
    }

    meshs.iter_mut().for_each(|m| m.setup());

    Ok(Model {
        name: String::new(),
        meshs,
    })
}

fn process_node<V: VertexData>(
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
) -> Option<Mesh<V>> {
    node.mesh().map(|mesh| {
        let mut indices: Vec<u32> = Vec::new();
        let mut raw = RawVertex::default();
        let mut textures = Vec::new();

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let gltf_material = primitive.material();

            // BIG TODO: REFACTOR TEXTURE LOADING INTO FUNCTIONS
            if let Some(normal_tex) = gltf_material.normal_texture() {
                if let Some(tex) = reader.read_tex_coords(normal_tex.tex_coord()) {
                    raw.tex_coords
                        .extend(tex.into_f32().map(|t| Vector2::from(t)));
                }
                // TODO implement From<gltf::Texture> for glboot::Texture
                let data = &images[normal_tex.texture().index()];
                textures.push((
                    "material.normal_tex".to_owned(),
                    Texture::from_bytes(
                        &data.pixels,
                        data.width as i32,
                        data.height as i32,
                        gl::RGBA, // TODO use the data specified format
                    )
                    .unwrap(),
                ));
            }

            if let Some(base_color) = gltf_material.pbr_metallic_roughness().base_color_texture() {
                if let Some(tex) = reader.read_tex_coords(base_color.tex_coord()) {
                    raw.tex_coords
                        .extend(tex.into_f32().map(|t| Vector2::from(t)));
                }

                let data = &images[base_color.texture().index()];
                textures.push((
                    "material.diffuse_tex".to_owned(),
                    Texture::from_bytes(
                        &data.pixels,
                        data.width as i32,
                        data.height as i32,
                        gl::RGBA, // TODO use the data specified format
                    )
                    .unwrap(),
                ));
            }

            if let Some(pos) = reader.read_positions() {
                raw.vertices.extend(pos.map(|p| Vector3::from(p)));
            }
            if let Some(norm) = reader.read_normals() {
                raw.normals.extend(norm.map(|n| Vector3::from(n)));
            }
            // if let Some(tex) = reader.read_tex_coords(0) {
            //     raw.tex_coords
            //         .extend(tex.into_f32().map(|t| Vector2::from(t)));
            // }
            // if let Some(tex) = reader.read_tex_coords(1) {
            //     tex_coords.extend(tex.into_f32().map(|t| Vector2::from(t)));
            // }

            if let Some(ind) = reader.read_indices() {
                indices.extend(ind.into_u32());
            }
        }

        Mesh::new(V::from_raw(raw), Some(Arc::new(textures)), indices, None)
    })
    // unimplemented!()
}
