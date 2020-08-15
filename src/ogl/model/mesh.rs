use std::{fmt::Debug, path::Path};

use crate::ogl::{
    buffers::{array::VertexArray, index::IndexBuffer, vertex::VertexBuffer},
    program::ShaderProgram,
    texture::Texture,
};

use super::VertexData;

use cgmath::{prelude::*, vec3, Matrix4, Vector3, Vector4};

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub base_color: Vector4<f32>,
    pub base_tex: Option<usize>,

    pub metallic: f32,
    pub roughness: f32,
    pub metallic_tex: Option<usize>,

    pub normal: Option<usize>,
    pub occlusion_tex: Option<usize>,
    pub occlusion_str: f32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            base_tex: None,
            metallic: 1.0,
            roughness: 0.0,
            metallic_tex: None,
            normal: None,
            occlusion_tex: None,
            occlusion_str: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct Mesh<V: VertexData> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
    pub material: Option<usize>,
    pub default_transform: cgmath::Matrix4<f32>,
    pub bounds: (Vector3<f32>, Vector3<f32>),
    vbo: VertexBuffer,
    ibo: IndexBuffer,
    vao: VertexArray,
}

impl<V: VertexData> Mesh<V> {
    pub fn new(
        vertices: Vec<V>,
        indices: Vec<u32>,
        material: Option<usize>,
        default_transform: cgmath::Matrix4<f32>,
        bounds: (Vector3<f32>, Vector3<f32>),
    ) -> Self {
        Mesh {
            vertices,
            indices,
            material,
            default_transform,
            bounds,
            vbo: VertexBuffer::default(),
            ibo: IndexBuffer::default(),
            vao: VertexArray::default(),
        }
    }

    pub fn setup(&mut self) {
        self.vbo = VertexBuffer::new(&self.vertices);
        self.ibo = IndexBuffer::new(&self.indices);
        self.vao = VertexArray::new();
        let layout = V::get_layout();

        self.vao.add_buffer(&self.vbo, &layout);
    }

    fn draw(&self, shader: &mut ShaderProgram, materials: &[Material]) {
        shader.set_uniform("default_model", self.default_transform);

        if let Some(mat_index) = self.material {
            let material = &materials[mat_index];

            shader.set_uniform("material.base_color", material.base_color);
            shader.set_uniform("material.has_base_color", 1);

            if let Some(base_tex_index) = material.base_tex {
                shader.set_uniform("material.base_tex", base_tex_index as i32);
                shader.set_uniform("material.has_base_tex", 1);
            } else {
                shader.set_uniform("material.has_base_tex", 0);
            }
        }

        self.vao.bind();
        self.ibo.bind();
        shader.send_uniforms();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            )
        };

        self.ibo.unbind();
        self.vao.unbind();
    }
}

#[derive(Debug)]
pub struct Model<V: VertexData> {
    pub name: String,
    pub meshs: Vec<Mesh<V>>,
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
    pub sphere: (Vector3<f32>, f32),
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
        let trans = Matrix4::from_translation(-self.sphere.0);
        let r_max = (self.sphere.0.distance(vec3(0.0, 0.0, 1.0)))
            * (45.0 * (std::f32::consts::PI / 180.0) / 2.0).sin();
        let scale = Matrix4::from_scale(r_max / self.sphere.1);

        shader.set_uniform("global", trans);
        shader.set_uniform("model", scale);

        for (i, tex) in self.textures.iter().enumerate() {
            tex.bind(i as u32);
        }

        for mesh in self.meshs.iter() {
            mesh.draw(shader, &self.materials);
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
        dbg!(self.sphere);
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

        let radius = min.distance(max) / 2.0;
        let center = (min + max) / 2.0;

        (center, radius)
    }
}
