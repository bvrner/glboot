use glfw::{Context, WindowHint};
use std::{
    ops::{Deref, DerefMut},
    sync::mpsc::Receiver,
};

pub struct Window {
    win: glfw::Window,
    pub events: Option<Receiver<(f64, glfw::WindowEvent)>>,
    pub width: u32,
    pub height: u32,
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
        glfw.window_hint(WindowHint::Samples(Some(4)));

        let (mut win, events) = glfw
            .create_window(dimensions.0, dimensions.1, name, glfw::WindowMode::Windowed)
            .unwrap();

        win.set_all_polling(true);
        win.set_sticky_keys(true);
        Window {
            win,
            events: Some(events),
            width: dimensions.0,
            height: dimensions.1,
        }
    }

    pub fn hidden() -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        glfw.window_hint(WindowHint::ContextVersion(3, 3));
        glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(WindowHint::Samples(Some(4)));
        glfw.window_hint(WindowHint::Visible(false));

        let (win, events) = glfw
            .create_window(1, 1, "", glfw::WindowMode::Windowed)
            .unwrap();

        Window {
            win,
            events: Some(events),
            width: 0,
            height: 0,
        }
    }

    #[inline]
    pub fn load_gl(&mut self) {
        gl::load_with(|s| self.win.get_proc_address(s) as *const _);
    }

    #[inline]
    pub fn update(&mut self) {
        self.win.glfw.poll_events();
        self.win.swap_buffers();
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
