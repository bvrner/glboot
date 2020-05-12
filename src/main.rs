mod ogl;
mod ui;
mod window;

use ogl::{
    buffers::{
        array::{Layout, VertexArray},
        index::IndexBuffer,
        vertex::VertexBuffer,
    },
    program::ShaderProgram,
    texture::Texture,
};
use window::Window;

use cgmath::Vector3;
use glfw::{self, Action, Context, Key};
use imgui;
use ui::ImguiGLFW;

fn main() {
    // shader and texture paths
    let root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    let v_path = format!("{}/shaders/vertex.glsl", root);
    let f_path = format!("{}/shaders/frag.glsl", root);
    let t_path = format!("{}/textures/wall.jpg", root);

    let mut window = Window::new("Bootstrap", (800, 600));
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
    imgui.set_ini_filename(None);
    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);

    let mut program = ShaderProgram::from_files(v_path, f_path, None).unwrap();
    let texture = Texture::new(t_path).unwrap();

    let vertices = [
        -0.5, 0.5, 0.0, 0.0, 1.0, -0.5, -0.5, 0.0, 0.0, 0.0, 0.5, -0.5, 0.0, 1.0, 0.0, 0.5, 0.5,
        0.0, 1.0, 1.0,
    ];
    let indices = [0, 1, 2, 0, 3, 2];
    let vao = VertexArray::new();
    let vbo = VertexBuffer::new::<f32>(&vertices);
    let ibo = IndexBuffer::new::<u8>(&indices);
    let layout = layout![(3, f32, gl::FLOAT), (2, f32, gl::FLOAT)];

    vao.add_buffer(&vbo, &layout);

    // imgui utils
    let mut colors: [f32; 3] = [1.0, 1.0, 1.0];
    let mut clicked = false;
    let mut mode = gl::FILL;

    program.set_uniform("tex", 0);
    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::PolygonMode(gl::FRONT_AND_BACK, mode);
        }

        texture.bind(0);
        program.bind();
        program.set_uniform("col", Vector3::from(colors));
        program.send_uniforms();
        ibo.bind();
        vao.bind();

        unsafe {
            // gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_BYTE, std::ptr::null());
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

                if ui.collapsing_header(imgui::im_str!("Options")).build() {
                    if ui.checkbox(imgui::im_str!("Wireframe"), &mut clicked) {
                        mode = if clicked { gl::LINE } else { gl::FILL };
                    }
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
