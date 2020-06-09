//TODO refactor this whole mess

use glboot::core::{arcball::ArcBall, camera::Camera, window::Window};
use glboot::ogl::{
    // buffers::{VertexArray, VertexBuffer},
    model::mesh::Model,
    model::StandardVertex,
    program::ShaderProgram,
    // texture::Texture,
};

use cgmath::{Matrix4, Point2, Point3, SquareMatrix, Vector3};
use glfw::{self, Action, Context, Key};

fn main() {
    // shader and texture paths
    let root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    let shader_path = format!("{}/shaders/flattex.glsl", root);
    // let m_path = format!("{}/models/matilda/scene.gltf", root);
    // let m_path = format!("{}/models/tests/Duck.gltf", root);
    let m_path = format!("{}/models/simpler_dragon.glb", root);

    let (mut window, mut imgui) = setup();

    let mut program = ShaderProgram::from_file(shader_path).unwrap();

    let model: Model<StandardVertex> = Model::load(m_path).unwrap();

    let mut gui_state = glboot::ImGuiState::default();
    let camera = Camera::new(Point3::new(0.0, 0.3, 0.5), Vector3::new(0.0, -0.3, -0.5));

    program.set_uniform(
        "projection",
        cgmath::perspective(cgmath::Deg(45.0_f32), 800.0 / 600.0, 0.1_f32, 100f32),
    );
    program.set_uniform("view", camera.get_matrix());
    program.set_uniform("model", Matrix4::from_scale(gui_state.scale));
    program.set_uniform("arc", Matrix4::identity());

    let mut arc = ArcBall::new(800.0, 600.0);
    let events = window.events.take().unwrap();
    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::PolygonMode(
                gl::FRONT_AND_BACK,
                if gui_state.wireframe {
                    gl::LINE
                } else {
                    gl::FILL
                },
            );
        }

        model.draw(&mut program);

        // if imgui.draw(&mut window, &mut gui_state) {
        //     program.set_uniform("col", Vector3::from(gui_state.colors));

        //     let (w, h) = window.get_framebuffer_size();
        //     let proj = cgmath::perspective(
        //         cgmath::Deg(gui_state.cam_slider),
        //         w as f32 / h as f32,
        //         0.1_f32,
        //         100f32,
        //     );

        //     program.set_uniform("refraction", gui_state.env as i32);
        //     program.set_uniform("projection", proj);
        //     program.set_uniform("model", Matrix4::from_scale(gui_state.scale));
        // }
        window.update();

        for (_, event) in glfw::flush_messages(&events) {
            imgui.handle_event(&event);

            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::MouseButton(glfw::MouseButtonRight, Action::Press, _) => {
                    let point = window.get_cursor_pos();
                    arc.click(Point2::new(point.0 as f32, point.1 as f32));
                }
                glfw::WindowEvent::MouseButton(glfw::MouseButtonRight, Action::Release, _) => {
                    arc.finish();
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    if arc.is_on {
                        let rotation = Matrix4::from(arc.drag(Point2::new(x as f32, y as f32)));
                        program.set_uniform("arc", rotation);
                    }
                }
                glfw::WindowEvent::FramebufferSize(w, h) => {
                    unsafe { gl::Viewport(0, 0, w, h) };

                    arc.update(w as f32, h as f32);

                    let proj = cgmath::perspective(
                        cgmath::Deg(gui_state.cam_slider),
                        w as f32 / h as f32,
                        0.1_f32,
                        100f32,
                    );

                    program.set_uniform("projection", proj);
                }
                _ => {}
            }
        }
    }
}

fn setup() -> (Window, glboot::ImGUI) {
    let mut window = Window::new("Bootstrap", (800, 600));
    window.make_current();
    window.load_gl();

    let imgui = glboot::ImGUI::new(&mut window);

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::Enable(gl::CULL_FACE);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Enable(gl::MULTISAMPLE);
    }

    (window, imgui)
}
