mod ogl;
mod ui;
mod window;

use ogl::{
    buffers::{
        array::{Layout, VertexArray},
        vertex::VertexBuffer,
    },
    program::ShaderProgram,
};
use window::Window;

use cgmath::Vector3;
use glfw::{self, Action, Context, Key};
use imgui;
use ui::ImguiGLFW;

fn main() {
    // Create a windowed mode window and its OpenGL context
    let mut window = Window::new("Bootstrap", (800, 600));
    // Make the window's context current
    window.make_current();
    window.load_gl();
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    let mut imgui = imgui::Context::create();
    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);

    let root = env!("CARGO_MANIFEST_DIR");
    let v_path = format!("{}/shadersrc/vertex.glsl", root);
    let f_path = format!("{}/shadersrc/frag.glsl", root);
    let mut program = ShaderProgram::from_files(v_path, f_path, None).unwrap();

    let vertices = [-0.5_f32, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

    let vao = VertexArray::new();
    let vbo = VertexBuffer::new(&vertices);
    let layout = layout![(3, f32, gl::FLOAT)];

    vao.add_buffer(&vbo, &layout);
    let mut colors: [f32; 3] = [1.0, 1.0, 1.0];

    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        program.bind();
        program.set_uniform("col", Vector3::from(colors));
        program.send_uniforms();
        vao.bind();

        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        let ui = imgui_glfw.frame(&mut window, &mut imgui);
        imgui::Window::new(imgui::im_str!("Playground"))
            .size([300.0, 300.0], imgui::Condition::Once)
            .build(&ui, || {
                if ui.collapsing_header(imgui::im_str!("Color")).build() {
                    imgui::ColorPicker::new(imgui::im_str!("Pick a Color"), &mut colors)
                        .alpha(false)
                        .display_rgb(true)
                        .build(&ui);
                }
            });
        imgui_glfw.draw(ui, &mut window);
        window.swap_buffers();

        window.process_events(|flow: &mut window::ControlFlow, event| {
            imgui_glfw.handle_event(&mut imgui, event);
            match *event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    *flow = window::ControlFlow::Quit;
                }
                glfw::WindowEvent::FramebufferSize(w, h) => unsafe { gl::Viewport(0, 0, w, h) },
                _ => {}
            }
        });
    }
}
