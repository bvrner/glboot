use super::mesh::{Mesh, Model, Vertex};
use cgmath::{vec2, vec3};

use std::fmt::Debug;
use std::path::Path;

pub fn load_obj<P>(path: P) -> Result<Model, String>
where
    P: AsRef<Path> + Debug,
{
    let (models, _materials) = tobj::load_obj(path, true).unwrap();
    let mut meshs = vec![];

    for model in models.iter() {
        let mesh = &model.mesh;
        let num_vertices = mesh.positions.len() / 3;

        // data to fill
        let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
        let indices: Vec<u32> = mesh.indices.clone();

        let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);
        for i in 0..num_vertices {
            vertices.push(Vertex {
                vertice: vec3(p[i * 3], p[i * 3 + 1], p[i * 3 + 2]),
                normal: vec3(n[i * 3], n[i * 3 + 1], n[i * 3 + 2]),
                tex_coords: vec2(t[i * 2], t[i * 2 + 1]),
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
