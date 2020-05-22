use super::mesh::{Material, Mesh, Model, Vertex};
use crate::ogl::texture::Texture;
use cgmath::{vec2, vec3};

use rayon::prelude::*;

use std::{
    error::Error,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

// TODO General error type for all handlers
#[derive(Debug, Clone)]
pub enum LoaderError {
    IOError,
    FileError,
}

pub fn load_obj<P>(path: P) -> Result<Model, String>
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

    let mut meshs: Vec<Mesh> = models
        .into_par_iter()
        .map(|model| {
            let mesh = model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            // data to fill
            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices: Vec<u32> = mesh.indices;

            let (p, n, t) = (mesh.positions, mesh.normals, mesh.texcoords);

            // I'm sure that there's a smarter way to do this
            // but all my approaches were slower in the benchs
            for i in 0..num_vertices {
                vertices.push(Vertex {
                    vertice: if !p.is_empty() {
                        vec3(p[i * 3], p[i * 3 + 1], p[i * 3 + 2])
                    } else {
                        vec3(0.0, 0.0, 0.0)
                    },
                    normal: if !n.is_empty() {
                        vec3(n[i * 3], n[i * 3 + 1], n[i * 3 + 2])
                    } else {
                        vec3(0.0, 0.0, 0.0)
                    },
                    tex_coords: if !t.is_empty() {
                        vec2(t[i * 2], t[i * 2 + 1])
                    } else {
                        vec2(0.0, 0.0)
                    },
                    tangent: vec3(0.0, 0.0, 0.0),
                    bitangent: vec3(0.0, 0.0, 0.0),
                })
            }

            let (material, texture) = if let Some(index) = mesh.material_id {
                (Some(materials[index]), Some(textures[index].clone()))
            } else {
                (None, None)
            };

            Mesh::new(vertices, texture, indices, material)
        })
        .collect();

    meshs.iter_mut().for_each(|m| m.setup());
    Ok(Model {
        name: String::new(),
        meshs,
    })
}
