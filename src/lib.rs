#[macro_use]
pub mod macros;
pub mod aabb;
pub mod core;
pub mod ogl;
pub mod scene;

use crate::core::ui::ImguiGLFW;
use imgui::{Context, Ui};

use ogl::program::ShaderProgram;
use std::{cell::RefCell, rc::Rc};

pub struct ImGUI {
    pub imgui: RefCell<imgui::Context>,
    pub imgui_glfw: ImguiGLFW,
    renders: Vec<Rc<RefCell<dyn ImRender>>>, // main_shader: Rc<RefCell<ShaderProgram>>, // TODO remove this
}

pub trait ImRender {
    fn render(&mut self, _: &Ui) {}
}

impl ImGUI {
    pub fn new(window: &mut glfw::Window) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        let imgui_glfw = ImguiGLFW::new(&mut imgui, window);

        ImGUI {
            imgui: RefCell::new(imgui),
            imgui_glfw,
            renders: Vec::new(),
        }
    }

    pub fn push_render(&mut self, r: Rc<RefCell<dyn ImRender>>) {
        self.renders.push(r);
    }

    #[inline]
    pub fn handle_event(&mut self, event: &glfw::WindowEvent) {
        self.imgui_glfw
            .handle_event(&mut self.imgui.borrow_mut(), event);
    }

    pub fn draw(&mut self, window: &mut glfw::Window) {
        let mut imgui = self.imgui.borrow_mut();
        let ui = self.imgui_glfw.frame(window, &mut imgui);

        imgui::Window::new(imgui::im_str!("Playground"))
            .size([300.0, 300.0], imgui::Condition::Once)
            .build(&ui, || {
                for r in self.renders.iter() {
                    r.borrow_mut().render(&ui);
                }
            });

        self.imgui_glfw.draw(ui, window);
    }
}
