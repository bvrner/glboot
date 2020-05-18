#[macro_use]
pub mod macros;
pub mod core;
pub mod ogl;

use crate::core::ui::ImguiGLFW;
use imgui::Context;

pub struct ImGUI {
    imgui: imgui::Context,
    imgui_glfw: ImguiGLFW,
    // pub state: Option<ImGuiState>,
}

#[derive(Debug, Copy, Clone)]
pub struct ImGuiState {
    pub colors: [f32; 3],
    pub wireframe: bool,
    pub cam_slider: f32,
    pub scale: f32,
}

impl Default for ImGuiState {
    fn default() -> ImGuiState {
        ImGuiState {
            colors: [0.5, 0.5, 0.5],
            wireframe: false,
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

        ImGUI {
            imgui,
            imgui_glfw,
            // state: Some(ImGuiState::default()),
        }
    }

    #[inline]
    pub fn handle_event(&mut self, event: &glfw::WindowEvent) {
        self.imgui_glfw.handle_event(&mut self.imgui, event);
    }

    pub fn draw(&mut self, window: &mut glfw::Window, state: &mut ImGuiState) {
        let ui = self.imgui_glfw.frame(window, &mut self.imgui);

        imgui::Window::new(imgui::im_str!("Playground"))
            .size([300.0, 300.0], imgui::Condition::Once)
            .build(&ui, || {
                if ui.collapsing_header(imgui::im_str!("Object")).build() {
                    color_picker(&ui, &mut state.colors);
                    scale(&ui, &mut state.scale);
                }
                options(&ui, &mut state.wireframe);
                camera(&ui, &mut state.cam_slider);
            });
        self.imgui_glfw.draw(ui, window);
    }
}

#[inline]
fn color_picker(ui: &imgui::Ui, colors: &mut [f32; 3]) {
    imgui::ColorPicker::new(imgui::im_str!("Pick a Color"), colors)
        .alpha(false)
        .display_rgb(true)
        .build(&ui);
}

#[inline]
fn options(ui: &imgui::Ui, clicked: &mut bool) {
    if ui.collapsing_header(imgui::im_str!("Options")).build() {
        ui.checkbox(imgui::im_str!("Wireframe"), clicked);
    }
}

#[inline]
fn camera(ui: &imgui::Ui, fov: &mut f32) {
    if ui.collapsing_header(imgui::im_str!("Camera")).build() {
        imgui::Slider::new(imgui::im_str!("FOV"), 0.1..=90.0).build(&ui, fov);
    }
}

#[inline]
fn scale(ui: &imgui::Ui, scale: &mut f32) {
    imgui::Slider::new(imgui::im_str!("Scale"), 0.1..=1.0).build(&ui, scale);
}
