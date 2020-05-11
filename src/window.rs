// use glfw::Context;
// use glfw::WindowHint;
use imgui_glfw_rs::glfw::{self, Context, WindowHint};
use std::sync::mpsc::Receiver;

pub struct Window {
    win: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ControlFlow {
    Continue,
    Quit,
}

impl Window {
    pub fn new(name: &str, dimensions: (u32, u32)) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        glfw.window_hint(WindowHint::ContextVersion(3, 3));
        glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        let (mut win, events) = glfw
            .create_window(dimensions.0, dimensions.1, name, glfw::WindowMode::Windowed)
            .unwrap();

        win.set_all_polling(true);
        Window { win, events }
    }

    #[inline]
    pub fn make_current(&mut self) {
        self.win.make_current();
    }

    #[inline]
    pub fn should_close(&self) -> bool {
        self.win.should_close()
    }

    #[inline]
    pub fn swap_buffers(&mut self) {
        self.win.swap_buffers();
    }

    #[inline]
    pub fn load_gl(&mut self) {
        gl::load_with(|s| self.win.get_proc_address(s) as *const _);
    }

    pub fn update(&mut self) {
        self.swap_buffers();
        self.win.glfw.poll_events();
    }

    pub fn process_events<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut ControlFlow, glfw::WindowEvent),
    {
        let mut flow = ControlFlow::Continue;
        for (_, event) in glfw::flush_messages(&self.events) {
            callback(&mut flow, event);
        }

        if flow == ControlFlow::Quit {
            self.win.set_should_close(true);
        }
    }
}

impl AsRef<glfw::Window> for Window {
    fn as_ref(&self) -> &glfw::Window {
        &self.win
    }
}

impl AsMut<glfw::Window> for Window {
    fn as_mut(&mut self) -> &mut glfw::Window {
        &mut self.win
    }
}
