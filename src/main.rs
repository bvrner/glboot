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
use glfw::{Action, Key};

fn main() {
    // Create a windowed mode window and its OpenGL context
    let mut window = Window::new("Win", (800, 600));
    // Make the window's context current
    window.make_current();
    window.load_gl();

    let root = env!("CARGO_MANIFEST_DIR");
    let v_path = format!("{}/shadersrc/vertex.glsl", root);
    let f_path = format!("{}/shadersrc/frag.glsl", root);
    let mut program = ShaderProgram::from_files(v_path, f_path, None).unwrap();

    let vertices = [-0.5_f32, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

    let vao = VertexArray::new();
    let vbo = VertexBuffer::new(&vertices);
    let layout = layout![(3, f32, gl::FLOAT)];

    vao.add_buffer(&vbo, &layout);

    program.set_uniform("col", Vector3::new(0.5, 0.2, 0.7));
    // Loop until the user closes the window
    while !window.should_close() {
        window.process_events(|flow: &mut window::ControlFlow, event| match event {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                *flow = window::ControlFlow::Quit;
            }
            glfw::WindowEvent::FramebufferSize(w, h) => unsafe { gl::Viewport(0, 0, w, h) },
            _ => {}
        });

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            program.bind();
            program.send_uniforms();
            vao.bind();

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.update();
    }
}
