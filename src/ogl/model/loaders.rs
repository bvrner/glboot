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

pub fn load_obj<P, V: VertexData + Send>(path: P) -> Result<Model<V>, String>
where
    P: AsRef<Path> + Debug,
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
