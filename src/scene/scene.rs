use crate::{
    aabb::Aabb,
    ogl::{buffers::*, material::Material, program::ShaderProgram, texture::Texture},
    ImRender,
};
use cgmath::{prelude::*, Matrix4, Quaternion, Vector3};
use thiserror::Error;

use super::{Mesh, Node, Primitive, Vertice};

use rayon::prelude::*;
use std::convert::TryFrom;
use std::path::Path;

#[derive(Debug)]
pub struct Scene {
    // use boxed slices instead?
    nodes: Vec<Node>,  // all nodes
    roots: Vec<usize>, // indices of the roots
    textures: Vec<Texture>,
    materials: Vec<Material>,
    pub aabb: Aabb,
    pub scale: f32,
    pub rotation: Quaternion<f32>,
    pub translation: Vector3<f32>,

    // Aabb rendering stuff
    // should this be part of the Aabb structure?
    vao_: VertexArray,
    vbo_: VertexBuffer,
    ibo_: IndexBuffer,
    draw_aabb: bool,
}

impl ImRender for Scene {
    fn render(&mut self, ui: &imgui::Ui) {
        if imgui::CollapsingHeader::new(imgui::im_str!("Scene")).build(&ui) {
            imgui::TreeNode::new(imgui::im_str!("m1"))
                .label(imgui::im_str!("Model"))
                .build(ui, || {
                    if imgui::Slider::new(imgui::im_str!("Scale"), 0.0001..=1.0)
                        .build(&ui, &mut self.scale)
                    {
                        if self.scale < 0.0001 {
                            self.scale = 0.1
                        }
                    }

                    if ui.small_button(imgui::im_str!("Reset Scale")) {
                        self.scale = 1.0;
                    }

                    let mut vec = self.translation.into();
                    if imgui::InputFloat3::new(ui, imgui::im_str!("Translation"), &mut vec).build()
                    {
                        self.translation = vec.into();
                    }

                    if ui.small_button(imgui::im_str!("Reset Trans.")) {
                        self.translation = Vector3::new(0.0, 0.0, 0.0);
                    }

                    // let mut vec = self.rotation.into();
                    // if imgui::InputFloat4::new(ui, imgui::im_str!("Rotation"), &mut vec).build() {
                    //     self.rotation = vec.into();
                    // }

                    // if ui.small_button(imgui::im_str!("Reset")) {
                    //     self.rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
                    // }

                    ui.checkbox(imgui::im_str!("AABB"), &mut self.draw_aabb);
                });
        }
    }
}

impl Scene {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, LoaderError> {
        load_gltf(path)
    }

    pub fn render(&self, shader: &mut ShaderProgram, aabb_shader: &mut ShaderProgram) {
        shader.bind();

        for (i, tex) in self.textures.iter().enumerate() {
            tex.bind(i as u32);
        }

        // TODO cache the transform
        // let transform = Matrix4::from_translation(self.translation)
        //     * Matrix4::from(self.rotation)
        //     * Matrix4::from_scale(self.scale);
        // TODO maybe a option to select the transform order?
        let transform = Matrix4::from(self.rotation)
            * Matrix4::from_translation(self.translation)
            * Matrix4::from_scale(self.scale);

        // start rendering by the roots which will render it's children and so on and so forth
        for &node in self.roots.iter() {
            self.nodes[node].draw(shader, &self.materials, &self.nodes, transform);
        }
        shader.unbind();

        if self.draw_aabb {
            aabb_shader.bind();

            aabb_shader.set_uniform("trans", transform);
            aabb_shader.send_uniforms();

            self.vao_.bind();
            self.ibo_.bind();

            // Aabb indices will always be the same, so it's safe to hardcode
            unsafe {
                gl::DrawElements(gl::LINE_LOOP, 4, gl::UNSIGNED_INT, std::ptr::null());
                gl::DrawElements(
                    gl::LINE_LOOP,
                    4,
                    gl::UNSIGNED_INT,
                    (4 * 4) as *const u32 as *const std::ffi::c_void,
                );
                gl::DrawElements(
                    gl::LINES,
                    8,
                    gl::UNSIGNED_INT,
                    (8 * 4) as *const u32 as *const std::ffi::c_void,
                );
            }

            aabb_shader.unbind();
            self.vao_.unbind();
            self.ibo_.unbind();
        }
    }

    fn gen_aabb(&mut self) {
        let mut aabb = Aabb::default();

        for &node in self.roots.iter() {
            let root_aabb = self.nodes[node].gen_aabb(&self.nodes, Matrix4::identity());
            aabb = aabb.surrounds(&root_aabb);
        }

        let (aabb_v, aabb_i) = aabb.gen_vertices();

        let vao_ = VertexArray::new();
        let layout = layout![(3, f32, gl::FLOAT)];
        let vbo_ = VertexBuffer::new(&aabb_v);
        let ibo_ = IndexBuffer::new(&aabb_i);

        vao_.add_buffer(&vbo_, &layout);

        self.aabb = aabb;
        self.vao_ = vao_;
        self.vbo_ = vbo_;
        self.ibo_ = ibo_;
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

    let textures: Result<Vec<Texture>, LoaderError> =
        images.into_iter().map(Texture::try_from).collect();
    let textures = textures?;

    let materials: Vec<Material> = document
        .materials()
        .into_iter()
        .par_bridge()
        .map(Material::from)
        .collect();

    let nodes: Vec<Node> = document
        .nodes()
        .map(|node| process_node(&buffers, &node))
        .collect();

    // this likely won't work for files with multiple scenes
    // but It's not a concern for now
    let mut roots = Vec::new();
    for scene in document.scenes() {
        for node in scene.nodes() {
            roots.push(node.index());
        }
    }

    let mut scene = Scene {
        roots,
        nodes,
        textures,
        materials,
        scale: 1.0,
        rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        translation: Vector3::new(0.0, 0.0, 0.0),
        aabb: Aabb::default(),
        vao_: VertexArray::default(),
        vbo_: VertexBuffer::default(),
        ibo_: IndexBuffer::default(),
        draw_aabb: false,
    };

    scene.gen_aabb();
    Ok(scene)
}

fn process_node(buffers: &[gltf::buffer::Data], node: &gltf::Node) -> Node {
    let mesh = node.mesh().map(|m| process_mesh(&buffers, &m));
    let transform = node.transform();
    let children = node.children().map(|child| child.index()).collect();

    Node::new(mesh, transform, children)
}

fn process_mesh(buffers: &[gltf::buffer::Data], m: &gltf::Mesh) -> Mesh {
    let primitives = m
        .primitives()
        .map(|primitive| {
            let mut positions = Vec::new();

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(pos) = reader.read_positions() {
                positions.extend(pos.map(|p| Vector3::from(p)));
            }

            //TODO? dynamic vertice types according to the data present?
            let mut vertices: Vec<Vertice> = positions
                .into_iter()
                .map(|p| Vertice {
                    pos: Vector3::from(p),
                    ..Vertice::default()
                })
                .collect();

            if let Some(norm) = reader.read_normals() {
                for (i, normal) in norm.enumerate() {
                    vertices[i].normal = normal.into();
                }
            }

            if let Some(tex) = reader.read_tex_coords(0) {
                for (i, coord) in tex.into_f32().enumerate() {
                    vertices[i].tex = coord.into();
                }
            }

            let indices = if let Some(ind) = reader.read_indices() {
                ind.into_u32().collect()
            } else {
                vec![]
            };
            let bounds = primitive.bounding_box().into();

            Primitive::setup(vertices, indices, primitive.material().index(), bounds)
        })
        .collect();

    Mesh::new(primitives)
}
