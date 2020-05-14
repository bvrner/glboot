use glboot::core::{
    camera::Camera,
    ui::ImguiGLFW,
    window::{self, Window},
};
use glboot::ogl::{model::mesh::Model, program::ShaderProgram, texture::Texture};

use cgmath::{Matrix4, Point3, Vector3};
use glfw::{self, Action, Context, Key};
use imgui;

fn main() {
    // shader and texture paths
    let root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    let v_path = format!("{}/shaders/vertex.glsl", root);
    let f_path = format!("{}/shaders/frag.glsl", root);
    let t_path = format!("{}/textures/wall.jpg", root);
    let m_path = format!("{}/models/cube.obj", root);
    println!("{}", root);

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
    let model = Model::load(m_path).unwrap();

    // imgui utils
    let mut colors: [f32; 3] = [1.0, 1.0, 1.0];
    let mut clicked = false;
    let mut mode = gl::FILL;

    let camera = Camera::new(Point3::new(0.0, 2.0, 1.0), Vector3::new(0.0, -0.5, -1.0));
    let mut fov = 45.0;
    program.set_uniform(
        "model",
        Matrix4::from_translation(Vector3::new(0.0, 0.0, -2.0)),
    );
    program.set_uniform(
        "projection",
        cgmath::perspective(cgmath::Deg(fov), 800.0 / 600.0, 0.1_f32, 100f32),
    );
    program.set_uniform("view", camera.get_matrix());
    program.set_uniform("tex", 0);
    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::PolygonMode(gl::FRONT_AND_BACK, mode);
        }

        texture.bind(0);
        program.set_uniform("col", Vector3::from(colors));

        model.draw(&program);
        let ui = imgui_glfw.frame(&mut window, &mut imgui);
        imgui::Window::new(imgui::im_str!("Playground"))
            .size([300.0, 300.0], imgui::Condition::Once)
            .build(&ui, || {
                if ui.collapsing_header(imgui::im_str!("Object")).build() {
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

                if ui.collapsing_header(imgui::im_str!("Camera")).build() {
                    if imgui::Slider::new(imgui::im_str!("FOV"), 0.1..=90.0).build(&ui, &mut fov) {
                        let (w, h) = window.get_framebuffer_size();
                        program.set_uniform(
                            "projection",
                            cgmath::perspective(
                                cgmath::Deg(fov),
                                w as f32 / h as f32,
                                0.1_f32,
                                100f32,
                            ),
                        );
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
                glfw::WindowEvent::FramebufferSize(w, h) => {
                    unsafe { gl::Viewport(0, 0, w, h) };

                    program.set_uniform(
                        "projection",
                        cgmath::perspective(cgmath::Deg(fov), w as f32 / h as f32, 0.1_f32, 100f32),
                    );
                }
                _ => {}
            }
        });
    }
}
