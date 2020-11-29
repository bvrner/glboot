//TODO refactor this whole mess

use std::{cell::RefCell, rc::Rc};

use glboot::{
    core::{arcball::ArcBall, camera::Camera, window::Window},
    ogl::{
        buffers::{FramebufferBuilder, VertexArray, VertexBuffer},
        program::ShaderProgram,
        shaders::ShaderError, // texture::Texture,
    },
    scene::Scene,
};

use cgmath::{Matrix4, Point2, Point3, SquareMatrix, Vector3};
use glfw::{self, Action, Context, Key};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // shader and texture paths
    let root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    let shader_path = format!("{}/shaders/flattex.glsl", root);
    // let shader_path = format!("{}/shaders/procedural/bricks.glsl", root);
    // let m_path = format!("{}/models/matilda/scene.gltf", root);
    let m_path = format!("{}/models/back/scene.gltf", root);
    // let m_path = format!("{}/models/tests/BoxTextured.gltf", root);
    // let m_path = format!("{}/models/dragon.glb", root);

    let mut window = setup();

    let program = Rc::new(RefCell::new(ShaderProgram::from_file(shader_path)?));
    let mut post_programs = load_post_shaders()?;
    // let post_program = Rc::new(RefCell::new(ShaderProgram::from_file(post_path)?));

    for program in post_programs.iter_mut() {
        program.set_uniform("screenTex", 0);
    }

    let mut imgui = glboot::ImGUI::new(&mut window, program.clone());
    let framebuffer = FramebufferBuilder::new(1366, 713)
        .with_depth()
        .with_stencil()
        .with_samples(4)
        .build()
        .unwrap();
    let intermediate = FramebufferBuilder::new(1366, 713).build().unwrap();

    // let mut model: Model<StandardVertex> = Model::load(m_path)?;
    let mut scene = Scene::load(m_path)?;
    let mut gui_state = glboot::ImGuiState::default();
    let mut camera = Camera::new(Point3::new(0.0, 0.0, 15.0), Vector3::new(0.0, 0.0, -1.0));

    let screen_quad = [
        -1.0_f32, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0,
        1.0, -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0,
    ];

    let quad_vbo = VertexBuffer::new(&screen_quad);
    let quad_vao = VertexArray::new();
    let layout = glboot::layout![(2, f32, gl::FLOAT), (2, f32, gl::FLOAT)];

    quad_vao.add_buffer(&quad_vbo, &layout);

    {
        let mut program = program.borrow_mut();
        program.set_uniform(
            "projection",
            cgmath::perspective(cgmath::Deg(45.0_f32), 1366.0 / 713.0, 0.1_f32, 100f32),
        );
        program.set_uniform("view", camera.get_matrix());
        // program.set_uniform("model", Matrix4::from_scale(0.1));
    }

    let mut arc = ArcBall::new(1366.0, 713.0);
    let events = window.events.take().unwrap();
    let mut last_frame = 0.0;

    while !window.should_close() {
        let current_frame = window.glfw.get_time() as f32;
        let delta_time = current_frame - last_frame;
        last_frame = current_frame;

        // first pass, render scene to texture
        framebuffer.bind();
        unsafe {
            gl::Viewport(0, 0, framebuffer.width, framebuffer.height);
            gl::Enable(gl::DEPTH_TEST);
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

        scene.render(&mut program.borrow_mut());
        // model.draw(&mut program.borrow_mut());
        framebuffer.unbind();
        // copy data from fbo to another, needed for anti-aliasing
        framebuffer.blit(&intermediate);

        // second pass, render that texture to the screen
        {
            // properly select the shader, since the effects done by kernels
            // share the same shader in the list
            // also if we are using the kernel shader we need to set which kernel to use
            let post_opt = gui_state.post_option;
            let post_program = if post_opt > 2 && post_opt < 6 {
                let program = &mut post_programs[3];
                program.set_uniform("option", (post_opt - 3) as i32);
                program
            } else {
                &mut post_programs[if post_opt < 2 { post_opt } else { 4 }]
            };

            post_program.bind();
            post_program.send_uniforms();
            intermediate.bind_textures(0);
            quad_vao.bind();
            unsafe {
                gl::Viewport(0, 0, window.width as i32, window.height as i32);
                gl::Disable(gl::DEPTH_TEST);
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
            intermediate.unbind_textures();
            post_program.unbind();
            quad_vao.unbind();
        }

        // the passing of the model is a ad hoc that will be removed when
        // I implement scene and renderer traits and/or structs
        imgui.draw(&mut window, &mut gui_state, &mut scene);

        window.update();

        for (_, event) in glfw::flush_messages(&events) {
            imgui.handle_event(&event);

            // THIS NEEDS A MAJOR REFACTOR
            let mut program = program.borrow_mut();
            handle_cam(&mut camera, &event, &mut program, delta_time);
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
                        let rotation = Matrix4::from(arc.drag(Point2::new(x as f32, y as f32)));
                        // model.rotation = rotation;
                    }
                }
                glfw::WindowEvent::FramebufferSize(w, h) => {
                    window.width = w as u32;
                    window.height = h as u32;

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
fn handle_cam(
    camera: &mut Camera,
    event: &glfw::WindowEvent,
    program: &mut ShaderProgram,
    delta: f32,
) {
    let cam_speed = delta * 2.5;
    match event {
        glfw::WindowEvent::Key(Key::W, _, _, _) => {
            camera.pos += cam_speed * camera.front;
            program.set_uniform("view", camera.get_matrix());
        }
        glfw::WindowEvent::Key(Key::Space, _, _, _) => {
            camera.pos += cam_speed * camera.up;
            program.set_uniform("view", camera.get_matrix());
        }
        glfw::WindowEvent::Key(Key::LeftShift, _, _, _) => {
            camera.pos -= cam_speed * camera.up;
            program.set_uniform("view", camera.get_matrix());
        }
        glfw::WindowEvent::Key(Key::S, _, _, _) => {
            camera.pos -= cam_speed * camera.front;
            program.set_uniform("view", camera.get_matrix());
        }
        _ => {}
    }
}

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
