use super::mesh::{Material, Mesh, Model};
use crate::ogl::texture::Texture;
use cgmath::{vec2, vec3, Vector2, Vector3};

use super::{RawVertex, VertexData};

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

pub fn load_obj<P, V>(path: P) -> Result<Model<V>, String>
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
        .map(|mat| {
            let metallic_roughness = mat.pbr_metallic_roughness();
            let base_color = metallic_roughness.base_color_factor().into();
            let base_tex = metallic_roughness
                .base_color_texture()
                .map(|info| info.texture().index());

            Material {
                base_color,
                base_tex,
                ..Default::default()
            }
        })
        .collect();

    let mut meshs = Vec::new();

    for scene in document.scenes() {
        for node in scene.nodes() {
            let mut stack = vec![node];

            while let Some(node) = stack.pop() {
                if let Some(mesh) = process_node(&node, &buffers) {
                    meshs.push(mesh);
                }

                for child in node.children() {
                    stack.push(child);
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
) -> Option<Mesh<V>> {
    let default_transform = node.transform().matrix().into();

    node.mesh().map(|mesh| {
        let mut indices: Vec<u32> = Vec::new();
        let mut raw = RawVertex::default();
        let mut material = None;

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            material = primitive.material().index();

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
        }

        // dbg!(&raw);

        Mesh::new(V::from_raw(raw), indices, material, default_transform)
    })
    // unimplemented!()
}
