use super::mesh::{Mesh, Model, Vertex};
use cgmath::{vec2, vec3};

use std::{error::Error, fmt::Debug, path::Path};

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
    let (models, _materials) = tobj::load_obj(path, true).unwrap();
    let mut meshs = vec![];

    for model in models.into_iter() {
        let mesh = model.mesh;
        let num_vertices = mesh.positions.len() / 3;

        // data to fill
        let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
        let indices: Vec<u32> = mesh.indices;

        let (p, n, t) = (mesh.positions, mesh.normals, mesh.texcoords);
        for i in 0..num_vertices {
            vertices.push(Vertex {
                vertice: vec3(
                    p.get(i * 3).copied().unwrap_or_default(),
                    p.get(i * 3 + 1).copied().unwrap_or_default(),
                    p.get(i * 3 + 2).copied().unwrap_or_default(),
                ),
                normal: vec3(
                    n.get(i * 3).copied().unwrap_or_default(),
                    n.get(i * 3 + 1).copied().unwrap_or_default(),
                    n.get(i * 3 + 2).copied().unwrap_or_default(),
                ),
                tex_coords: vec2(
                    t.get(i * 2).copied().unwrap_or_default(),
                    t.get(i * 2 + 1).copied().unwrap_or_default(),
                ),
            });
        }

        // process material
        // let mut textures = Vec::new();
        // if let Some(material_id) = mesh.material_id {
        //     let material = &materials[material_id];

        //     // 1. diffuse map
        //     if !material.diffuse_texture.is_empty() {
        //         let texture =
        //             self.loadMaterialTexture(&material.diffuse_texture, "texture_diffuse");
        //         textures.push(texture);
        //     }
        //     // 2. specular map
        //     if !material.specular_texture.is_empty() {
        //         let texture =
        //             self.loadMaterialTexture(&material.specular_texture, "texture_specular");
        //         textures.push(texture);
        //     }
        //     // 3. normal map
        //     if !material.normal_texture.is_empty() {
        //         let texture = self.loadMaterialTexture(&material.normal_texture, "texture_normal");
        //         textures.push(texture);
        //     }
        //     // NOTE: no height maps
        // }

        meshs.push(Mesh::new(vertices, vec![], indices));
    }

    Ok(Model {
        name: String::new(),
        meshs,
    })
}
