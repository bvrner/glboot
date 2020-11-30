#[macro_use]
pub mod macros;
pub mod core;
pub mod ogl;
pub mod scene;

use crate::core::ui::ImguiGLFW;
use imgui::{im_str, Context, ImString, Ui};

use ogl::program::ShaderProgram;
use std::{cell::RefCell, rc::Rc};

pub struct ImGUI {
    pub imgui: RefCell<imgui::Context>,
    pub imgui_glfw: ImguiGLFW,
    renders: Vec<RefCell<Box<dyn ImRender>>>, // main_shader: Rc<RefCell<ShaderProgram>>, // TODO remove this
}

// This is a pretty wack way to deal with options
// I'll have to rethink it later
#[derive(Debug, Clone)]
pub struct ImGuiState {
    pub bg_color: [f32; 4],
    pub wireframe: bool,
    pub env: bool,
    pub cam_slider: f32,
    pub scale: f32,
    pub post_option: usize,
}

impl Default for ImGuiState {
    fn default() -> ImGuiState {
        ImGuiState {
            bg_color: [0.1, 0.1, 0.1, 0.1],
            wireframe: false,
            env: true,
            cam_slider: 45.0,
            scale: 0.1,
            post_option: 0,
        }
    }
}

pub trait ImRender {
    fn render(&mut self, _: &Ui) {}
}

impl ImGUI {
    pub fn new(
        window: &mut glfw::Window,
        // main_shader: Rc<RefCell<ShaderProgram>>,
        // post_shader: Rc<RefCell<ShaderProgram>>,
    ) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        let imgui_glfw = ImguiGLFW::new(&mut imgui, window);

        ImGUI {
            imgui: RefCell::new(imgui),
            imgui_glfw,
            renders: Vec::new()
            // main_shader,
            // post_shader,
        }
    }

    pub fn push_render(&mut self, r: RefCell<Box<dyn ImRender>>) {
        self.renders.push(r);
    }

    #[inline]
    pub fn handle_event(&mut self, event: &glfw::WindowEvent) {
        self.imgui_glfw
            .handle_event(&mut self.imgui.borrow_mut(), event);
    }

    pub fn draw(
        &mut self,
        window: &mut glfw::Window,
        // state: &mut ImGuiState,
        // scene: &mut crate::scene::Scene,
    ) {
        let mut imgui = self.imgui.borrow_mut();
        let ui = self.imgui_glfw.frame(window, &mut imgui);

        imgui::Window::new(imgui::im_str!("Playground"))
            .size([300.0, 300.0], imgui::Condition::Once)
            .build(&ui, || {
                for r in self.renders.iter() {
                    r.borrow_mut().render(&ui);
                }
                // if imgui::CollapsingHeader::new(imgui::im_str!("Object")).build(&ui) {
                //     if imgui::Slider::new(imgui::im_str!("Scale"), 0.000000001..=1.0)
                //         .build(&ui, &mut state.scale)
                //     {
                //         scene.transform = cgmath::Matrix4::from_scale(state.scale)
                //         // model.scale = cgmath::Matrix4::from_scale(state.scale);
                //         // TODO bugged: this transform should be given to the model, not directly to the shader
                //         // self.main_shader
                //         //     .borrow_mut()
                //         //     .set_uniform("model", cgmath::Matrix4::from_scale(state.scale));
                //     }

                //     ui.checkbox(imgui::im_str!("Wireframe"), &mut state.wireframe);
                // }
                // if imgui::CollapsingHeader::new(imgui::im_str!("Options")).build(&ui) {
                //     if imgui::ColorEdit::new(imgui::im_str!("Clear color"), &mut state.bg_color)
                //         .build(&ui)
                //     {
                //         unsafe {
                //             gl::ClearColor(
                //                 state.bg_color[0],
                //                 state.bg_color[1],
                //                 state.bg_color[2],
                //                 state.bg_color[3],
                //             );
                //         }
                //     }
                // }

                // if imgui::CollapsingHeader::new(imgui::im_str!("Camera")).build(&ui) {
                //     if imgui::Slider::new(imgui::im_str!("FOV"), 0.1..=90.0)
                //         .build(&ui, &mut state.cam_slider)
                //     {
                //         let (w, h) = window.get_framebuffer_size();
                //         let proj = cgmath::perspective(
                //             cgmath::Deg(state.cam_slider),
                //             w as f32 / h as f32,
                //             0.1_f32,
                //             100f32,
                //         );
                //         self.main_shader
                //             .borrow_mut()
                //             .set_uniform("projection", proj);
                //     }
                // }

                // if imgui::CollapsingHeader::new(im_str!("Post-Processing")).build(&ui) {
                //     const NAMES: [&'static str; 7] = [
                //         "None",
                //         "Negative",
                //         "Black and White",
                //         "Sharp",
                //         "Blur",
                //         "Edge",
                //         "Sobel",
                //     ];

                //     for (i, name) in NAMES.iter().enumerate() {
                //         if imgui::Selectable::new(&ImString::new(name.to_owned()))
                //             .selected(state.post_option == i)
                //             .build(&ui)
                //         {
                //             state.post_option = i;

                //             println!("{}", i);
                //         }
                //     }
                // }
                // ui.separator();
                // ui.text(
                //     "Use WASD to move camera (WIP).\
                //          \nRight click and mouse to rotate object (kinda broken in some meshs).\
                //          \nSpace to go up and Shift to go down.\
                //          \nR to reset the rotation.",
                // );
            });

        self.imgui_glfw.draw(ui, window);
    }
}
