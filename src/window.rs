// use glfw::Context;
// use glfw::WindowHint;
use glfw::WindowHint;
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    sync::mpsc::Receiver,
};

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
        win.set_sticky_keys(true);
        Window { win, events }
    }

    #[inline]
    pub fn load_gl(&mut self) {
        gl::load_with(|s| self.win.get_proc_address(s) as *const _);
    }

    pub fn process_events<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut ControlFlow, &glfw::WindowEvent),
    {
        self.win.glfw.poll_events();

        let mut flow = ControlFlow::Continue;
        for (_, event) in glfw::flush_messages(&self.events) {
            callback(&mut flow, &event);
        }

        if flow == ControlFlow::Quit {
            self.set_should_close(true);
        }
    }
}

impl Deref for Window {
    type Target = glfw::Window;

    fn deref(&self) -> &Self::Target {
        &self.win
    }
}

impl DerefMut for Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.win
    }
}
