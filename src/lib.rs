#[macro_use]
pub mod macros;
pub mod core;
pub mod ogl;

use crate::core::ui::ImguiGLFW;
use imgui::Context;

pub struct ImGUI {
    pub imgui: imgui::Context,
    pub imgui_glfw: ImguiGLFW,
}

#[derive(Debug, Copy, Clone)]
pub struct ImGuiState {
    pub colors: [f32; 3],
    pub wireframe: bool,
    pub env: bool,
    pub cam_slider: f32,
    pub scale: f32,
}

impl Default for ImGuiState {
    fn default() -> ImGuiState {
        ImGuiState {
            colors: [0.5, 0.5, 0.5],
            wireframe: false,
            env: true,
            cam_slider: 45.0,
            scale: 0.1,
        }
    }
}

impl ImGUI {
    pub fn new(window: &mut glfw::Window) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        let imgui_glfw = ImguiGLFW::new(&mut imgui, window);

        ImGUI { imgui, imgui_glfw }
    }

    #[inline]
    pub fn handle_event(&mut self, event: &glfw::WindowEvent) {
        self.imgui_glfw.handle_event(&mut self.imgui, event);
    }

    pub fn draw(&mut self, window: &mut glfw::Window, state: &mut ImGuiState) -> bool {
        let ui = self.imgui_glfw.frame(window, &mut self.imgui);
        let mut updated = false;

        imgui::Window::new(imgui::im_str!("Playground"))
            .size([300.0, 300.0], imgui::Condition::Once)
            .build(&ui, || {
                if imgui::CollapsingHeader::new(imgui::im_str!("Object")).build(&ui) {
                    // updated |= color_picker(&ui, &mut state.colors);
                    updated |= scale(&ui, &mut state.scale);
                    updated |= env_option(&ui, &mut state.env);
                }
                updated |= options(&ui, &mut state.wireframe);
                updated |= camera(&ui, &mut state.cam_slider);
            });
        self.imgui_glfw.draw(ui, window);
        updated
    }

    pub fn is_mouse_down(&mut self, window: &mut glfw::Window, button: imgui::MouseButton) -> bool {
        let ui = self.imgui_glfw.frame(window, &mut self.imgui);

        ui.is_mouse_down(button)
    }
}

// #[inline]
// fn color_picker(ui: &imgui::Ui, colors: &mut [f32; 3]) -> bool {
//     imgui::ColorPicker::new(imgui::im_str!("Pick a Color"), colors)
//         .alpha(false)
//         .display_rgb(true)
//         .build(&ui)
// }

#[inline]
fn env_option(ui: &imgui::Ui, clicked: &mut bool) -> bool {
    ui.checkbox(imgui::im_str!("Refraction|Reflection"), clicked)
}

#[inline]
fn options(ui: &imgui::Ui, clicked: &mut bool) -> bool {
    if imgui::CollapsingHeader::new(imgui::im_str!("Options")).build(ui) {
        let mut color = [0.1, 0.1, 0.1, 0.1];
        if imgui::ColorEdit::new(imgui::im_str!("Background color"), &mut color).build(ui) {
            unsafe {
                gl::ClearColor(color[0], color[1], color[2], color[3]);
            }
        }
        ui.checkbox(imgui::im_str!("Wireframe"), clicked)
    } else {
        false
    }
}

#[inline]
fn camera(ui: &imgui::Ui, fov: &mut f32) -> bool {
    if imgui::CollapsingHeader::new(imgui::im_str!("Camera")).build(ui) {
        imgui::Slider::new(imgui::im_str!("FOV"), 0.1..=90.0).build(&ui, fov)
    } else {
        false
    }
}

#[inline]
fn scale(ui: &imgui::Ui, scale: &mut f32) -> bool {
    imgui::Slider::new(imgui::im_str!("Scale"), 0.000000001..=1.0).build(&ui, scale)
}
