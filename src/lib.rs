#[macro_use]
pub mod macros;
pub mod aabb;
pub mod core;
pub mod ogl;
pub mod scene;

use crate::core::ui::ImguiGLFW;
use imgui::{Context, Ui};

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

// cgmath's slerp is bugged
pub fn slerp(
    left: cgmath::Quaternion<f32>,
    right: cgmath::Quaternion<f32>,
    amount: f32,
) -> cgmath::Quaternion<f32> {
    let num2;
    let num3;
    let num = amount;
    let mut num4 = (((left.v.x * right.v.x) + (left.v.y * right.v.y)) + (left.v.z * right.v.z))
        + (left.s * right.s);
    let mut flag = false;
    if num4 < 0.0 {
        flag = true;
        num4 = -num4;
    }
    if num4 > 0.999_999 {
        num3 = 1.0 - num;
        num2 = if flag { -num } else { num };
    } else {
        let num5 = num4.acos();
        let num6 = 1.0 / num5.sin();
        num3 = ((1.0 - num) * num5).sin() * num6;
        num2 = if flag {
            -(num * num5).sin() * num6
        } else {
            (num * num5).sin() * num6
        };
    }
    cgmath::Quaternion::new(
        (num3 * left.s) + (num2 * right.s),
        (num3 * left.v.x) + (num2 * right.v.x),
        (num3 * left.v.y) + (num2 * right.v.y),
        (num3 * left.v.z) + (num2 * right.v.z),
    )
}
