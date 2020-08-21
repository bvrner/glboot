use std::{fmt::Debug, path::Path};

use crate::ogl::{program::ShaderProgram, texture::Texture};

use super::{Material, Mesh, VertexData};

use cgmath::{prelude::*, vec3, Matrix4, Vector3};

#[derive(Debug)]
pub struct Model<V: VertexData> {
    pub name: String,
    pub meshs: Vec<Mesh<V>>,
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
    pub sphere: (Vector3<f32>, f32),
    pub global: Matrix4<f32>,
}

impl<V: VertexData + Send> Model<V> {
    pub fn load<P>(path: P) -> Result<Self, String>
    where
        P: AsRef<Path> + Debug,
    {
        let mut model = match path.as_ref().extension() {
            Some(ext) if ext == "obj" => super::loaders::load_obj(path)?,
            Some(ext) if ext == "gltf" => super::loaders::load_gltf(path)?,
            Some(ext) if ext == "glb" => super::loaders::load_gltf(path)?,
            _ => return Err(String::from("Unsuported file format")),
        };

        model.calculate_bounding_sphere();
        Ok(model)
    }

    pub fn draw(&self, shader: &mut ShaderProgram) {
        shader.bind();

        for (i, tex) in self.textures.iter().enumerate() {
            tex.bind(i as u32);
        }

        for mesh in self.meshs.iter() {
            mesh.draw(shader, &self.materials, self.global);
        }
        shader.unbind();
    }

    // see https://www.researchgate.net/publication/242453691_An_Efficient_Bounding_Sphere
    fn calculate_bounding_sphere(&mut self) {
        let (mut sphere_center, mut radius) = self.find_initial_sphere();
        let mut rad_sq = radius * radius;

        for mesh in self.meshs.iter() {
            let (min, max) = mesh.bounds;

            let (min_distance, max_distance) = (min - sphere_center, max - sphere_center);
            let (min_mag, max_mag) = (min_distance.magnitude2(), max_distance.magnitude2());

            if min_mag > rad_sq {
                radius = (radius + min_mag) / 2.0;

                rad_sq = radius * radius;
                let old_to_new = min_mag - radius;

                sphere_center = (radius * sphere_center + old_to_new * min) / min_mag;
            }

            if max_mag > rad_sq {
                radius = (radius + max_mag) / 2.0;

                rad_sq = radius * radius;
                let old_to_new = max_mag - radius;

                sphere_center = (radius * sphere_center + old_to_new * max) / max_mag;
            }
        }

        self.sphere = (sphere_center, radius);
        let scale =
            self.sphere.0.distance(Vector3::new(0.0, 0.0, 15.0)) * (45f32.to_radians() / 2.0).sin();
        self.global = Matrix4::from_scale(scale / self.sphere.1);
        // dbg!(self.sphere);
    }

    fn find_initial_sphere(&self) -> (Vector3<f32>, f32) {
        let bounds = {
            let min = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
            let max = vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

            (min, max)
        };

        let (min, max) = self
            .meshs
            .iter()
            .fold(bounds, |(bound_min, bound_max), mesh| {
                let (mesh_min, mesh_max) = mesh.bounds;

                // let mesh_min = mesh.default_transform * mesh_min.extend(1.0);
                // let mesh_max = mesh.default_transform * mesh_max.extend(1.0);

                let minx = bound_min.x.min(mesh_min.x);
                let miny = bound_min.y.min(mesh_min.y);
                let minz = bound_min.z.min(mesh_min.z);

                let maxx = bound_max.x.max(mesh_max.x);
                let maxy = bound_max.y.max(mesh_max.y);
                let maxz = bound_max.z.max(mesh_max.z);

                let vmin = vec3(minx, miny, minz);
                let vmax = vec3(maxx, maxy, maxz);

                (vmin, vmax)
            });
        // dbg!((min, max));

        let radius = min.distance(max) / 2.0;
        let center = (min + max) / 2.0;

        (center, radius)
    }
}
