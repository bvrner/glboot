use super::{buffers::*, program::ShaderProgram};
use crate::{scene::Scene, ImRender};

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

        scene.render(&mut self.main, aabb_program);

        // model.draw(&mut program.borrow_mut());
        self.front.unbind();
        // copy data from fbo to another, needed for anti-aliasing
        self.front.blit(&self.int);

        // second pass, render that texture to the screen
        {
            // properly select the shader, since the effects done by kernels
            // share the same shader in the list
            // also if we are using the kernel shader we need to set which kernel to use
            // let post_opt = gui_state.post_option;
            // let post_program = if post_opt > 2 && post_opt < 6 {
            //     let program = &mut post_programs[3];
            //     program.set_uniform("option", (post_opt - 3) as i32);
            //     program
            // } else {
            //     &mut post_programs[if post_opt < 2 { post_opt } else { 4 }]
            // };

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
                        .label(imgui::im_str!("Front"))
                        .push(ui)
                    {
                        ui.text("WIP");
                        front.pop(ui);
                    }

                    if let Some(front) = imgui::TreeNode::new(imgui::im_str!("r2.2"))
                        .label(imgui::im_str!("Back"))
                        .push(ui)
                    {
                        ui.text("WIP");
                        front.pop(ui);
                    }
                });
        }
    }
}
