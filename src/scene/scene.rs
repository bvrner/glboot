use crate::{
    aabb::Aabb,
    ogl::{buffers::*, material::Material, program::ShaderProgram, texture::Texture},
    ImRender,
};
use cgmath::{prelude::*, Matrix4, Quaternion, Vector3};
use thiserror::Error;

use super::{
    animations::{Animation, Animations, Mode},
    skin::Skin,
    Mesh, Node, Primitive, Vertice,
};

// use rayon::prelude::*;
use std::convert::TryFrom;
use std::path::Path;

#[derive(Debug)]
pub struct Scene {
    // use boxed slices instead?
    pub nodes: Vec<Node>,  // all nodes
    pub roots: Vec<usize>, // indices of the roots

    node_parent: Vec<(usize, Option<usize>)>, // (node, parent) indices for traversal
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
    pub animations: Animations,
    pub skins: Vec<Skin>,

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
    // other options
    // anim_index: Option<usize>,
}

impl Scene {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, LoaderError> {
        load_gltf(path)
    }

    pub fn update(&mut self, time: f32) {
        self.animations.animate(time, &mut self.nodes); // {
        let this_transform = Matrix4::from_translation(self.translation)
            * Matrix4::from(self.rotation)
            * Matrix4::from_scale(self.scale);
        // let this_transform = Matrix4::from(self.rotation)
        //     * Matrix4::from_translation(self.translation)
        //     * Matrix4::from_scale(self.scale);

        for (node, parent) in self.node_parent.iter() {
            let parent_transform = parent.map_or(this_transform, |p_index| {
                self.nodes[p_index].global_transform
            });

            self.nodes[*node].update_global(parent_transform);
        }
        // }
    }

    fn initial_setup(&mut self) {
        // first generate AABB
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

        // set the global transform of the nodes
        let this_transform = Matrix4::from_translation(self.translation)
            * Matrix4::from(self.rotation)
            * Matrix4::from_scale(self.scale);

        for (node, parent) in self.node_parent.iter() {
            let parent_transform = parent.map_or(this_transform, |p_index| {
                self.nodes[p_index].global_transform
            });

            self.nodes[*node].update_global(parent_transform);
        }
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
        // .par_bridge()
        .map(Material::from)
        .collect();

    let nodes: Vec<Node> = document
        .nodes()
        .map(|node| process_node(&buffers, &node))
        .collect();

    let skins = document
        .skins()
        .map(|s| Skin::from_gltf(&s, &buffers))
        .collect();

    // this likely won't work for files with multiple scenes
    // but It's not a concern for now
    let mut roots = Vec::new();
    for scene in document.scenes() {
        for node in scene.nodes() {
            roots.push(node.index());
        }
    }

    let node_parent = super::node::build_tree(&nodes, &roots);
    let animations = document
        .animations()
        .map(|anim| Animation::new(&anim, &buffers))
        .collect();

    let mut scene = Scene {
        roots,
        nodes,
        textures,
        materials,
        skins,
        animations: Animations::new(animations),
        scale: 1.0,
        rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        translation: Vector3::new(0.0, 0.0, 0.0),
        aabb: Aabb::default(),
        vao_: VertexArray::default(),
        vbo_: VertexBuffer::default(),
        ibo_: IndexBuffer::default(),
        draw_aabb: false,
        node_parent,
        // anim_index: None,
    };

    scene.initial_setup();
    Ok(scene)
}

fn process_node(buffers: &[gltf::buffer::Data], node: &gltf::Node) -> Node {
    let mesh = node.mesh().map(|m| process_mesh(&buffers, &m));
    let transform = node.transform();
    let children = node.children().map(|child| child.index()).collect();
    let skin = node.skin().map(|s| s.index());

    Node::new(mesh, transform, children, skin)
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

            if let Some(joints) = reader.read_joints(0) {
                for (i, j) in joints.into_u16().enumerate() {
                    vertices[i].joints =
                        cgmath::Vector4::new(j[0] as f32, j[1] as f32, j[2] as f32, j[3] as f32);
                }
            }

            if let Some(w) = reader.read_weights(0) {
                for (i, w) in w.into_f32().enumerate() {
                    vertices[i].weights = cgmath::Vector4::from(w);
                }
            }

            if let Some(t) = reader.read_tangents() {
                for (i, t) in t.enumerate() {
                    vertices[i].tangent = t.into();
                }
            }

            let indices = if let Some(ind) = reader.read_indices() {
                ind.into_u32().collect()
            } else {
                vec![]
            };
            let bounds = primitive.bounding_box().into();

            Primitive::setup(
                vertices,
                indices,
                primitive.material().index(),
                bounds,
                primitive.mode().as_gl_enum(),
            )
        })
        .collect();

    Mesh::new(primitives, m.name().map(String::from))
}

impl ImRender for Scene {
    fn render(&mut self, ui: &imgui::Ui) {
        if imgui::CollapsingHeader::new(imgui::im_str!("Scene")).build(&ui) {
            imgui::TreeNode::new(imgui::im_str!("m1"))
                .label(imgui::im_str!("Model"))
                .build(ui, || {
                    if let Some(t_node) = imgui::TreeNode::new(imgui::im_str!("m1.1"))
                        .label(imgui::im_str!("Transformations"))
                        .push(ui)
                    {
                        if imgui::Slider::new(imgui::im_str!("Scale"))
                            .range(0.0001..=2.0)
                            .build(&ui, &mut self.scale)
                        {
                            if self.scale < 0.0001 {
                                self.scale = 0.1
                            }
                        }

                        let bid = ui.push_id("Reset");

                        // ui.same_line(0.0);
                        if ui.small_button(imgui::im_str!("Reset")) {
                            self.scale = 1.0;
                        }

                        bid.pop(ui);

                        let mut vec = self.translation.into();
                        if imgui::InputFloat3::new(ui, imgui::im_str!("Translation"), &mut vec)
                            .build()
                        {
                            self.translation = vec.into();
                        }

                        let bid = ui.push_id("Reset");
                        // ui.same_line(0.0);
                        if ui.small_button(imgui::im_str!("Reset")) {
                            self.translation = Vector3::new(0.0, 0.0, 0.0);
                        }

                        bid.pop(ui);
                        // let mut vec = self.rotation.into();
                        // if imgui::InputFloat4::new(ui, imgui::im_str!("Rotation"), &mut vec).build() {
                        //     self.rotation = vec.into();
                        // }

                        // if ui.small_button(imgui::im_str!("Reset")) {
                        //     self.rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
                        // }
                        //
                        t_node.pop(ui);
                    }

                    // TODO move this elsewhere, maybe at the animations structure
                    if let Some(a_node) = imgui::TreeNode::new(imgui::im_str!("m1.2"))
                        .label(imgui::im_str!("Animations"))
                        .push(ui)
                    {
                        let preview = match self.animations.mode {
                            Mode::All => imgui::im_str!("{}", "All"),
                            Mode::None => imgui::im_str!("{}", "None"),
                            Mode::Single(i) => imgui::im_str!("{}", self.animations.inner[i].name),
                        };

                        if let Some(combo) = imgui::ComboBox::new(imgui::im_str!("Selected"))
                            .preview_value(&preview)
                            .begin(ui)
                        {
                            for (i, anim) in self.animations.inner.iter().enumerate() {
                                let is_selected = self.animations.mode == i;

                                if imgui::Selectable::new(&imgui::im_str!("{}", &anim.name))
                                    .selected(is_selected)
                                    .build(ui)
                                {
                                    self.animations.mode = Mode::Single(i);
                                }

                                if is_selected {
                                    ui.set_item_default_focus();
                                }
                            }

                            let all = self.animations.mode == Mode::All;

                            if imgui::Selectable::new(imgui::im_str!("All"))
                                .selected(all)
                                .build(ui)
                            {
                                self.animations.mode = Mode::All;
                            }

                            if all {
                                ui.set_item_default_focus();
                            }

                            let none = self.animations.mode == Mode::None;

                            if imgui::Selectable::new(imgui::im_str!("None"))
                                .selected(none)
                                .build(ui)
                            {
                                self.animations.mode = Mode::None;
                            }

                            if none {
                                ui.set_item_default_focus();
                            }

                            combo.end(ui);
                        }

                        let b_label = if self.animations.paused {
                            imgui::im_str!("Play")
                        } else {
                            imgui::im_str!("Pause")
                        };

                        self.animations.paused ^= ui.button(b_label, [40.0, 22.0]);
                        // ui.same_line(0.0);

                        // if ui.button(imgui::im_str!("Reset"), [40.0, 22.0]) {
                        //     self.animations.reset();
                        // }

                        a_node.pop(ui);
                    }

                    if let Some(o_node) = imgui::TreeNode::new(imgui::im_str!("m1.3"))
                        .label(imgui::im_str!("Other"))
                        .push(ui)
                    {
                        ui.checkbox(imgui::im_str!("AABB"), &mut self.draw_aabb);
                        o_node.pop(ui)
                    }
                });
        }
    }
}
