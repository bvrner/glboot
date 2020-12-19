use cgmath::{Matrix, Matrix4, SquareMatrix};

use super::{buffers::*, material::Material, program::ShaderProgram};
use crate::{
    scene::{skin::Skin, Mesh, Node, Scene},
    ImRender,
};

const SCREEN_QUAD: [f32; 24] = [
    -1.0_f32, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0,
    -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0,
];

#[derive(Debug)]
pub struct Renderer {
    screen: Screen,
    front: Framebuffer,
    int: Framebuffer,
    pub main: ShaderProgram,
    pub post: ShaderProgram,

    //general options
    p_mode: gl::types::GLenum,
    bg_col: [f32; 3],
}

#[derive(Debug)]
struct Screen {
    vao: VertexArray,
    vbo: VertexBuffer,
}

impl Renderer {
    pub fn create(w: i32, h: i32, main: ShaderProgram, post: ShaderProgram) -> Self {
        let vao = VertexArray::new();
        let vbo = VertexBuffer::new(&SCREEN_QUAD);
        let layout = layout![(2, f32, gl::FLOAT), (2, f32, gl::FLOAT)];

        vao.add_buffer(&vbo, &layout);

        let screen = Screen { vao, vbo };

        let front = FramebufferBuilder::new(w, h)
            .with_depth()
            .with_stencil()
            .with_samples(4)
            .build()
            .unwrap();

        let int = FramebufferBuilder::new(w, h).build().unwrap();

        Self {
            screen,
            front,
            int,
            main,
            post,
            p_mode: gl::FILL,
            bg_col: [0.0, 0.0, 0.0],
        }
    }

    pub fn render(&mut self, scene: &Scene, aabb_program: &mut ShaderProgram) {
        self.front.bind();
        unsafe {
            gl::Viewport(0, 0, self.front.width, self.front.height);
            gl::Enable(gl::DEPTH_TEST);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::PolygonMode(gl::FRONT_AND_BACK, self.p_mode);
        }

        // scene.render(&mut self.main, aabb_program);
        self.main.bind();
        let nodes = &scene.nodes;
        let roots = &scene.roots;
        let skins = &scene.skins;
        let materials = &scene.materials;
        let textures = &scene.textures;

        for (i, tex) in textures.iter().enumerate() {
            tex.bind(i as u32);
        }

        for root in roots.iter() {
            self.render_node(&nodes[*root], skins, materials, nodes);
        }

        self.main.unbind();
        // model.draw(&mut program.borrow_mut());
        self.front.unbind();
        // copy data from fbo to another, needed for anti-aliasing
        self.front.blit(&self.int);

        // second pass, render that texture to the screen
        {
            self.post.bind();
            self.post.send_uniforms();
            self.int.bind_textures(0);
            self.screen.vao.bind();

            unsafe {
                gl::Viewport(0, 0, self.int.width as i32, self.int.height as i32);
                gl::Disable(gl::DEPTH_TEST);
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }

            self.int.unbind_textures();
            self.post.unbind();
            self.screen.vao.unbind();
        }
    }

    fn render_node(&mut self, this: &Node, skins: &[Skin], materials: &[Material], nodes: &[Node]) {
        if let Some(ref mesh) = this.mesh {
            if let Some(skin) = this.skin {
                let joints = &skins[skin].joints;
                let joint_matrices: Vec<Matrix4<f32>> = joints
                    .iter()
                    .map(|joint| {
                        this.global_transform.invert().unwrap()
                            * nodes[joint.node].global_transform
                            * joint.bind_matrix
                    })
                    .collect();

                // dbg!(joint_matrices.len());

                unsafe {
                    let name = std::ffi::CString::new("joints").unwrap();
                    gl::UniformMatrix4fv(
                        gl::GetUniformLocation(self.main.0, name.as_ptr()),
                        joint_matrices.len() as i32,
                        gl::FALSE,
                        joint_matrices[0].as_ptr(),
                    );
                }
            }

            self.main.set_uniform("model", this.global_transform);
            self.render_mesh(mesh, materials);
        }

        for child in this.children.iter() {
            self.render_node(&nodes[*child], skins, materials, nodes);
        }
    }

    fn render_mesh(&mut self, mesh: &Mesh, materials: &[Material]) {
        let main = &mut self.main;

        for prim in mesh.primitives.iter() {
            if let Some(mat_index) = prim.material {
                let material = &materials[mat_index];

                main.set_uniform("material.base_color", material.base_color);
                main.set_uniform("material.has_base_color", 1);

                if let Some(base_tex_index) = material.base_tex {
                    main.set_uniform("material.base_tex", base_tex_index as i32);
                    main.set_uniform("material.has_base_tex", 1);
                } else {
                    main.set_uniform("material.has_base_tex", 0);
                }
            } else {
                // shader.set_uniform("material.base_color", material.base_color);
                main.set_uniform("material.has_base_color", 0);

                // shader.set_uniform("material.base_tex", base_tex_index as i32);
                main.set_uniform("material.has_base_tex", 0);
            }

            // main.set_uniform("model", transform);

            prim.vao.bind();
            prim.ibo.bind();
            main.send_uniforms();

            // TODO instancing
            unsafe {
                if prim.indices_count > 0 {
                    gl::DrawElements(
                        prim.mode,
                        prim.indices_count,
                        gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );
                } else {
                    gl::DrawArrays(prim.mode, 0, prim.vertice_count);
                }
            };

            prim.ibo.unbind();
            prim.vao.unbind();
        }
    }

    #[inline]
    pub fn resize(&mut self, w: i32, h: i32) {
        self.front = FramebufferBuilder::new(w, h)
            .with_depth()
            .with_stencil()
            .with_samples(4)
            .build()
            .unwrap();

        self.int = FramebufferBuilder::new(w, h).build().unwrap();
    }
}

impl ImRender for Renderer {
    fn render(&mut self, ui: &imgui::Ui) {
        if imgui::CollapsingHeader::new(imgui::im_str!("Renderer")).build(ui) {
            imgui::TreeNode::new(imgui::im_str!("r1"))
                .label(imgui::im_str!("Color"))
                .build(ui, || {
                    if imgui::ColorPicker::new(imgui::im_str!("BG Color"), &mut self.bg_col)
                        .build(ui)
                    {
                        unsafe {
                            gl::ClearColor(self.bg_col[0], self.bg_col[1], self.bg_col[2], 1.0);
                        }
                    }
                });
            imgui::TreeNode::new(imgui::im_str!("r2"))
                .label(imgui::im_str!("Shaders"))
                .build(ui, || {
                    if let Some(front) = imgui::TreeNode::new(imgui::im_str!("r2.1"))
                        .label(imgui::im_str!("First pass"))
                        .push(ui)
                    {
                        ui.text("WIP");
                        front.pop(ui);
                    }

                    if let Some(back) = imgui::TreeNode::new(imgui::im_str!("r2.2"))
                        .label(imgui::im_str!("Second pass"))
                        .push(ui)
                    {
                        ui.text("WIP");
                        back.pop(ui);
                    }
                });
        }
    }
}
