// use glfw::Context;
// use glfw::WindowHint;
use glfw::{Context, WindowHint};
use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::mpsc::Receiver,
};

pub struct Window {
    // if glfw is droped before the window we will panic
    pub glfw: ManuallyDrop<glfw::Glfw>,
    win: ManuallyDrop<glfw::Window>,
    pub events: Option<Receiver<(f64, glfw::WindowEvent)>>,
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
            win: ManuallyDrop::new(win),
            glfw: ManuallyDrop::new(glfw),
            events: Some(events),
        }
    }

    #[inline]
    pub fn load_gl(&mut self) {
        gl::load_with(|s| self.win.get_proc_address(s) as *const _);
    }

    #[inline]
    pub fn update(&mut self) {
        self.glfw.poll_events();
        self.win.swap_buffers();
    }

    // pub fn process_events<F>(&mut self, mut callback: F)
    // where
    //     F: FnMut(&glfw::WindowEvent) -> ControlFlow,
    // {
    //     self.glfw.poll_events();

    //     let mut flow = ControlFlow::Continue;
    //     for (_, event) in glfw::flush_messages(&self.events) {
    //         flow = callback(&event);
    //     }

    //     if flow == ControlFlow::Quit {
    //         self.set_should_close(true);
    //     }
    // }
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

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.win);
            ManuallyDrop::drop(&mut self.glfw);
        }
    }
}
