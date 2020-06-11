use super::mesh::{Material, Mesh, Model};
use crate::ogl::texture::Texture;
use cgmath::{Matrix4, Vector2, Vector3};

use super::{RawVertex, VertexData};
use rayon::prelude::*;

use std::{
    // error::Error,
    fmt::Debug,
    path::Path,
};

// TODO General error type for all handlers
#[derive(Debug, Clone)]
pub enum LoaderError {
    IOError,
    FileError,
}

// TODO rewrite this importer supporting the new vertex and material designs
pub fn load_obj<P, V>(_path: P) -> Result<Model<V>, String>
where
    P: AsRef<Path> + Debug,
    V: VertexData + Send,
{
    // let (models, materials) = tobj::load_obj(&path, true).unwrap();

    // let (materials, textures) = materials.into_iter().fold(
    //     (vec![], vec![]), // confusing, I know, sorry
    //     |(mut vec, mut texts), mat| {
    //         vec.push(Material {
    //             specular: mat.specular.into(),
    //             diffuse: mat.diffuse.into(),
    //             ambient: mat.ambient.into(),
    //             shininess: mat.shininess,
    //         });

    //         let mut text_vec = Vec::with_capacity(3);
    //         let dir = path.as_ref().parent().unwrap().to_str().unwrap();

    //         // 1. diffuse map
    //         if !mat.diffuse_texture.is_empty() {
    //             text_vec.push((
    //                 "material.diffuse".to_owned(),
    //                 Texture::new(&format!("{}/{}", dir, mat.diffuse_texture), false).unwrap(),
    //             ));
    //         }
    //         // 2. specular map
    //         if !mat.specular_texture.is_empty() {
    //             text_vec.push((
    //                 "material.specular".to_owned(),
    //                 Texture::new(&format!("{}/{}", dir, mat.specular_texture), false).unwrap(),
    //             ));
    //         }

    //         // normal map
    //         if !mat.normal_texture.is_empty() {
    //             text_vec.push((
    //                 "material.normal".to_owned(),
    //                 Texture::new(&format!("{}/{}", dir, mat.normal_texture), false).unwrap(),
    //             ));
    //         }
    //         texts.push(Arc::new(text_vec));
    //         (vec, texts)
    //     },
    // );

    // let mut meshs: Vec<Mesh<V>> = models
    //     .into_par_iter()
    //     .map(|model| {
    //         let mesh = model.mesh;

    //         let indices: Vec<u32> = mesh.indices;

    //         let raw = {
    //             let (p, n, t) = (mesh.positions, mesh.normals, mesh.texcoords);

    //             let v: Vec<Vector3<f32>> =
    //                 p.chunks_exact(3).map(|p| vec3(p[0], p[1], p[2])).collect();
    //             let n: Vec<Vector3<f32>> =
    //                 n.chunks_exact(3).map(|p| vec3(p[0], p[1], p[2])).collect();
    //             let t: Vec<Vector2<f32>> = t.chunks_exact(2).map(|p| vec2(p[0], p[1])).collect();

    //             RawVertex {
    //                 vertices: v,
    //                 normals: n,
    //                 tex_coords: t,
    //             }
    //         };

    //         let (material, texture) = if let Some(index) = mesh.material_id {
    //             (Some(materials[index]), Some(textures[index].clone()))
    //         } else {
    //             (None, None)
    //         };

    //         Mesh::new(V::from_raw(raw), texture, indices, material)
    //     })
    //     .collect();

    // // OpenGL doesn't like when we do it inside the parallel iterator
    // meshs.iter_mut().for_each(|m| m.setup());

    // Ok(Model {
    //     name: String::new(),
    //     meshs,
    // })

    unimplemented!()
}

pub fn load_gltf<P, V>(path: P) -> Result<Model<V>, String>
where
    P: AsRef<Path>,
    V: VertexData,
{
    let (document, buffers, images) = gltf::import(path).unwrap();

    assert_eq!(buffers.len(), document.buffers().count());
    assert_eq!(images.len(), document.images().count());

    let textures: Vec<Texture> = images
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
                _ => unimplemented!(),
            };

            unsafe {
                Texture::from_bytes(&data.pixels, data.width as i32, data.height as i32, format)
            }
            .unwrap()
        })
        .collect();

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

    let mut meshs = Vec::new();
    for scene in document.scenes() {
        for node in scene.nodes() {
            // each primitive must be transformed by the accum of
            // trannsformations from the root node till the local node
            // that's my quick ad hoc for that, I'll try to refactor it soon
            let root_transform = node.transform().matrix().into();
            let mut stack: Vec<(gltf::Node, Matrix4<f32>)> = vec![(node, root_transform)];

            while let Some((node, transform)) = stack.pop() {
                if let Some(mesh) = process_node(&node, &buffers, transform) {
                    meshs.extend(mesh);
                }

                for child in node.children() {
                    let local: Matrix4<f32> = child.transform().matrix().into();
                    stack.push((child, transform * local));
                }
            }
        }
    }

    meshs.iter_mut().for_each(|m| m.setup());

    Ok(Model {
        name: String::new(),
        meshs,
        textures,
        materials,
    })
}

fn process_node<V: VertexData>(
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    transform: Matrix4<f32>,
) -> Option<Vec<Mesh<V>>> {
    let node_transform: Matrix4<f32> = node.transform().matrix().into();
    let node_transform = node_transform * transform;

    node.mesh().map(|mesh| {
        mesh.primitives()
            .map(|primitive| {
                let mut indices: Vec<u32> = Vec::new();
                let mut raw = RawVertex::default();
                let material = primitive.material().index();

                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                if let Some(pos) = reader.read_positions() {
                    raw.vertices.extend(pos.map(|p| Vector3::from(p)));
                }
                if let Some(norm) = reader.read_normals() {
                    raw.normals.extend(norm.map(|n| Vector3::from(n)));
                }
                if let Some(tex) = reader.read_tex_coords(0) {
                    raw.tex_coords
                        .extend(tex.into_f32().map(|t| Vector2::from(t)));
                }

                if let Some(ind) = reader.read_indices() {
                    indices.extend(ind.into_u32());
                }

                Mesh::new(V::from_raw(raw), indices, material, node_transform)
            })
            .collect()
    })
}
