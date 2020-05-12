mod ogl;
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
use imgui_glfw_rs::glfw::{self, Action, Key};
use imgui_glfw_rs::imgui;
use imgui_glfw_rs::ImguiGLFW;

fn main() {
    // Create a windowed mode window and its OpenGL context
    let mut window = Window::new("Win", (800, 600));
    // Make the window's context current
    window.make_current();
    window.load_gl();
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        // gl::Enable(gl::DEPTH_TEST);
        // gl::DepthFunc(gl::LESS);
        // gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    let mut imgui = imgui::Context::create();
    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, window.as_mut());

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
    // Loop until the user closes the window
    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            program.bind();
            program.set_uniform("col", Vector3::from(colors));
            program.send_uniforms();
            vao.bind();

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        let ui = imgui_glfw.frame(window.as_mut(), &mut imgui);

        imgui::Window::new(&ui, imgui::im_str!("Playground"))
            .size([300.0, 300.0], imgui::Condition::Once)
            .build(|| {
                if ui.collapsing_header(imgui::im_str!("Color")).build() {
                    ui.color_picker(
                        imgui::im_str!("Pick a Color"),
                        imgui::EditableColor::Float3(&mut colors),
                    )
                    .alpha(false)
                    .rgb(true)
                    .build();
                }
            });
        imgui_glfw.draw(ui, window.as_mut());

        window.update();
        window.process_events(|flow: &mut window::ControlFlow, event| {
            imgui_glfw.handle_event(&mut imgui, &event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    *flow = window::ControlFlow::Quit;
                }
                glfw::WindowEvent::FramebufferSize(w, h) => unsafe { gl::Viewport(0, 0, w, h) },
                _ => {}
            }
        });
    }
}
