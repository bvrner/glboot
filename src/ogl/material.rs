use cgmath::Vector4;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub base_color: Vector4<f32>,
    pub base_tex: Option<usize>,

    pub metallic: f32,
    pub roughness: f32,
    pub metallic_tex: Option<usize>,

    pub normal: Option<usize>,
    pub occlusion_tex: Option<usize>,
    pub occlusion_str: f32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            base_tex: None,
            metallic: 1.0,
            roughness: 0.0,
            metallic_tex: None,
            normal: None,
            occlusion_tex: None,
            occlusion_str: 1.0,
        }
    }
}
