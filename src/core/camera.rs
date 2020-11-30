use crate::ImRender;
use cgmath::{InnerSpace, Matrix4, Point3, Vector3};

const WORLD_UP: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

/// A primitive camera
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub pos: Point3<f32>,
    pub front: Vector3<f32>,
    pub up: Vector3<f32>,
    pub right: Vector3<f32>,
    pub fov: f32,
}

impl Camera {
    /// Creates a new camera at `pos` looking at the direction `front`
    pub fn new(pos: Point3<f32>, front: Vector3<f32>) -> Self {
        Self::with_up(pos, front, WORLD_UP)
    }

    /// Creates a new camera with a custom worldwide up vector
    pub fn with_up(pos: Point3<f32>, front: Vector3<f32>, world_up: Vector3<f32>) -> Self {
        let (norm_up, norm_front) = (world_up.normalize(), front.normalize());
        let right = norm_up.cross(norm_front).normalize();
        let up = front.cross(right).normalize();

        Camera {
            pos,
            front: norm_front,
            up,
            right,
            fov: 45.0,
        }
    }

    /// Returns the view matrix of this camera
    #[inline]
    pub fn get_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at(self.pos, self.pos + self.front, self.up)
    }

    // Just a small helper
    #[inline]
    pub fn get_projection(&self, w: f32, h: f32) -> Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(self.fov), w as f32 / h as f32, 0.1_f32, 100f32)
    }
}

impl ImRender for Camera {
    fn render(&mut self, ui: &imgui::Ui) {
        if imgui::CollapsingHeader::new(imgui::im_str!("Camera")).build(&ui) {
            if imgui::Slider::new(imgui::im_str!("FOV"), 0.1..=90.0).build(&ui, &mut self.fov) {}
        }
    }
}
