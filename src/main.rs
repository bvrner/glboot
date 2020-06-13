//TODO refactor this whole mess

use glboot::core::{arcball::ArcBall, camera::Camera, window::Window};
use glboot::ogl::{
    buffers::{Framebuffer, VertexArray, VertexBuffer},
    model::mesh::Model,
    model::StandardVertex,
    program::ShaderProgram,
    // texture::Texture,
};

use cgmath::{Matrix4, Point2, Point3, SquareMatrix, Vector3};
use glfw::{self, Action, Context, Key};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // shader and texture paths
    let root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    // let shader_path = format!("{}/shaders/flattex.glsl", root);
    let shader_path = format!("{}/shaders/basic_ads.glsl", root);
    let post_path = format!("{}/shaders/postprocessing.glsl", root);
    // let m_path = format!("{}/models/matilda/scene.gltf", root);
    let m_path = format!("{}/models/simpler_dragon.glb", root);

    let (mut window, mut imgui) = setup();

    let mut program = ShaderProgram::from_file(shader_path)?;
    let mut post_program = ShaderProgram::from_file(post_path)?;
    post_program.set_uniform("screenTex", 0);

    let framebuffer = Framebuffer::new(800, 600).unwrap();

    let model: Model<StandardVertex> = Model::load(m_path).unwrap();

    let mut gui_state = glboot::ImGuiState::default();
    let mut camera = Camera::new(Point3::new(0.0, 0.3, 0.3), Vector3::new(0.0, -0.3, -0.3));

    let screen_quad = [
        -1.0_f32, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0,
        1.0, -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0,
    ];

    let quad_vbo = VertexBuffer::new(&screen_quad);
    let quad_vao = VertexArray::new();
    let layout = glboot::layout![(2, f32, gl::FLOAT), (2, f32, gl::FLOAT)];

    quad_vao.add_buffer(&quad_vbo, &layout);

    program.set_uniform(
        "projection",
        cgmath::perspective(cgmath::Deg(45.0_f32), 800.0 / 600.0, 0.1_f32, 100f32),
    );
    program.set_uniform("view", camera.get_matrix());
    program.set_uniform("model", Matrix4::from_scale(0.1));
    program.set_uniform("arc", Matrix4::identity());

    let mut arc = ArcBall::new(800.0, 600.0);
    let events = window.events.take().unwrap();
    while !window.should_close() {
        framebuffer.bind();
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
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
        framebuffer.unbind();

        post_program.bind();
        post_program.send_uniforms();
        framebuffer.bind_texture(0);
        quad_vao.bind();
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
        framebuffer.unbind_texture();
        post_program.unbind();
        quad_vao.unbind();

        if imgui.draw(&mut window, &mut gui_state) {
            let (w, h) = window.get_framebuffer_size();
            let proj = cgmath::perspective(
                cgmath::Deg(gui_state.cam_slider),
                w as f32 / h as f32,
                0.1_f32,
                100f32,
            );

            program.set_uniform("refraction", gui_state.env as i32);
            program.set_uniform("projection", proj);
            program.set_uniform(
                "model",
                Matrix4::from_scale(gui_state.scale)
                    * Matrix4::from_translation(Vector3::new(0.0, -0.5, 0.0)),
            );
            post_program.set_uniform("option", gui_state.post_option);
        }

        window.update();

        for (_, event) in glfw::flush_messages(&events) {
            imgui.handle_event(&event);

            // THIS NEEDS A MAJOR REFACTOR
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Key(Key::R, _, Action::Press, _) => {
                    arc.reset();
                    program.set_uniform("arc", Matrix4::identity());
                }
                glfw::WindowEvent::Key(Key::W, _, Action::Press, _) => {
                    camera.pos += 2.5 * camera.front;
                    program.set_uniform("view", camera.get_matrix());
                }
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    camera.pos += 2.5 * camera.up;
                    program.set_uniform("view", camera.get_matrix());
                }
                glfw::WindowEvent::Key(Key::LeftShift, _, Action::Press, _) => {
                    camera.pos -= 2.5 * camera.up;
                    program.set_uniform("view", camera.get_matrix());
                }
                glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
                    camera.pos -= 2.5 * camera.front;
                    program.set_uniform("view", camera.get_matrix());
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
                    framebuffer.update_dimensions(w, h);

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
    Ok(())
}

fn setup() -> (Window, glboot::ImGUI) {
    let mut window = Window::new("Bootstrap", (800, 600));
    window.make_current();
    window.load_gl();

    let imgui = glboot::ImGUI::new(&mut window);

    unsafe {
        // gl::Enable(gl::BLEND);
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
