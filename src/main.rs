//TODO refactor this whole mess
// this file a full blown ad hoc that I write just to test stuff out

use gl::types::*;
use std::{cell::RefCell, rc::Rc};

use glboot::{
    core::{arcball::ArcBall, camera::Camera, window::Window},
    ogl::{
        buffers::{FramebufferBuilder, VertexArray, VertexBuffer},
        program::ShaderProgram,
        renderer::Renderer,
        shaders::ShaderError, // texture::Texture,
    },
    scene::Scene,
};

use cgmath::{Point2, Point3, Vector3};
use glfw::{self, Action, Context, Key};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // shader and texture paths
    let root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    let shader_path = format!("{}/shaders/flattex.glsl", root);
    // let shader_path = format!("{}/shaders/procedural/bricks.glsl", root);
    let m_path = format!("{}/models/matilda/scene.gltf", root);
    // let m_path = format!("{}/models/back/scene.gltf", root);
    // let m_path = format!("{}/models/tests/BoxTextured.gltf", root);
    // let m_path = format!("{}/models/dragon.glb", root);

    let mut window = setup();

    unsafe {
        if gl::DebugMessageCallback::is_loaded() {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(callback), std::ptr::null());
        }
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::QUADS);
    }

    let program = ShaderProgram::from_file(shader_path)?;
    let mut pprogram = ShaderProgram::from_file(format!("{}/shaders/post/flat_post.glsl", root))?;
    pprogram.set_uniform("screenTex", 0);

    let mut renderer = Renderer::create(1366, 720, program, pprogram);
    let mut imgui = glboot::ImGUI::new(&mut window);

    let scene = Scene::load(m_path)?;
    let scene = RefCell::new(scene);
    let scene = Rc::new(scene);

    imgui.push_render(scene.clone());

    let gui_state = glboot::ImGuiState::default();
    let camera = Camera::new(Point3::new(0.0, 0.0, 15.0), Vector3::new(0.0, 0.0, -1.0));
    let camera = Rc::new(RefCell::new(camera));

    imgui.push_render(camera.clone());

    let screen_quad = [
        -1.0_f32, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0,
        1.0, -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0,
    ];

    let quad_vbo = VertexBuffer::new(&screen_quad);
    let quad_vao = VertexArray::new();
    let layout = glboot::layout![(2, f32, gl::FLOAT), (2, f32, gl::FLOAT)];

    quad_vao.add_buffer(&quad_vbo, &layout);

    {
        // let mut program = program.borrow_mut();
        renderer
            .main
            .set_uniform("projection", camera.borrow().get_projection(1366.0, 713.0));
        renderer
            .main
            .set_uniform("view", camera.borrow().get_matrix());
        // program.set_uniform("model", Matrix4::from_scale(0.1));
    }

    let mut aabb_program =
        ShaderProgram::from_sources(glboot::aabb::SOURCE_V, glboot::aabb::SOURCE_F, None)?;

    let mut arc = ArcBall::new(1366.0, 713.0);
    let events = window.events.take().unwrap();
    let mut last_frame = 0.0;

    while !window.should_close() {
        let current_frame = window.glfw.get_time() as f32;
        let delta_time = current_frame - last_frame;
        last_frame = current_frame;

        renderer.render(&scene.borrow(), &mut aabb_program);

        imgui.draw(&mut window);

        {
            // let mut program = program.borrow_mut();
            renderer.main.set_uniform(
                "projection",
                camera
                    .borrow()
                    .get_projection(window.width as f32, window.height as f32),
            );
        }

        window.update();

        for (_, event) in glfw::flush_messages(&events) {
            imgui.handle_event(&event);

            // THIS NEEDS A MAJOR REFACTOR
            // let mut program = program.borrow_mut();
            // handle_cam(&mut camera.borrow_mut(), &event, &mut program, delta_time);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Key(Key::R, _, Action::Press, _) => {
                    arc.reset();
                    // model.rotation = Matrix4::identity();
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
                        let rotation = arc.drag(Point2::new(x as f32, y as f32));
                        let mut scene = scene.borrow_mut();

                        scene.rotation = rotation;

                        // model.rotation = rotation;
                    }
                }
                // glfw::WindowEvent::FramebufferSize(w, h) => {
                //     window.width = w as u32;
                //     window.height = h as u32;

                //     arc.update(w as f32, h as f32);

                //     let proj = cgmath::perspective(
                //         cgmath::Deg(gui_state.cam_slider),
                //         w as f32 / h as f32,
                //         0.1_f32,
                //         100f32,
                //     );

                //     program.set_uniform("projection", proj);
                // }
                _ => {}
            }
        }
    }
    Ok(())
}

fn setup() -> Window {
    let mut window = Window::new("Bootstrap", (1366, 713));
    window.make_current();
    window.load_gl();

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

    window
}

// TODO update Camera struct to handle this
// fn handle_cam(
//     camera: &mut Camera,
//     event: &glfw::WindowEvent,
//     program: &mut ShaderProgram,
//     delta: f32,
// ) {
//     let cam_speed = delta * 2.5;
//     match event {
//         glfw::WindowEvent::Key(Key::W, _, _, _) => {
//             camera.pos += cam_speed * camera.front;
//             program.set_uniform("view", camera.get_matrix());
//         }
//         glfw::WindowEvent::Key(Key::Space, _, _, _) => {
//             camera.pos += cam_speed * camera.up;
//             program.set_uniform("view", camera.get_matrix());
//         }
//         glfw::WindowEvent::Key(Key::LeftShift, _, _, _) => {
//             camera.pos -= cam_speed * camera.up;
//             program.set_uniform("view", camera.get_matrix());
//         }
//         glfw::WindowEvent::Key(Key::S, _, _, _) => {
//             camera.pos -= cam_speed * camera.front;
//             program.set_uniform("view", camera.get_matrix());
//         }
//         _ => {}
//     }
// }

fn load_post_shaders() -> Result<Vec<ShaderProgram>, ShaderError> {
    let root = format!("{}/assets/shaders/post", env!("CARGO_MANIFEST_DIR"));

    Ok(vec![
        ShaderProgram::from_file(format!("{}/flat_post.glsl", root))?,
        ShaderProgram::from_file(format!("{}/negative.glsl", root))?,
        ShaderProgram::from_file(format!("{}/bw.glsl", root))?,
        ShaderProgram::from_file(format!("{}/kernel.glsl", root))?,
        ShaderProgram::from_file(format!("{}/sobel.glsl", root))?,
    ])
}
extern "system" fn callback(
    source: GLenum,
    gltype: GLenum,
    _id: GLuint,
    _severity: GLenum,
    _length: GLsizei,
    message: *const GLchar,
    _: *mut std::ffi::c_void,
) {
    use std::ffi::CStr;

    let source = match source {
        gl::DEBUG_SOURCE_API => "OpenGL API.",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window-system API.",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader compiler.",
        gl::DEBUG_SOURCE_THIRD_PARTY => "Third party application.",
        gl::DEBUG_SOURCE_APPLICATION => "User.",
        _ => "Other.",
    };

    let ty = match gltype {
        gl::DEBUG_TYPE_ERROR => "Error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior.",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior.",
        gl::DEBUG_TYPE_PORTABILITY => "Unportable functionality.",
        gl::DEBUG_TYPE_PERFORMANCE => "Possible performance issue.",
        gl::DEBUG_TYPE_MARKER => "Command stream annotation.",
        _ => "Other.",
    };

    let c_str = unsafe { CStr::from_ptr(message) };

    if let Ok(c_str) = c_str.to_str() {
        eprintln!(
            "OpenGL Log:\n\tSource: {}\n\tType: {}\n\tMessage: {}\n\n",
            source, ty, c_str
        );
    } else {
        eprintln!("OpenGL Error: Couldn't convert message to string.");
    }
}
