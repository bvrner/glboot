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

impl<'a> From<gltf::Material<'a>> for Material {
    fn from(mat: gltf::Material) -> Self {
        let metallic_roughness = mat.pbr_metallic_roughness();
        let base_color = metallic_roughness.base_color_factor().into();
        let base_tex = metallic_roughness
            .base_color_texture()
            .map(|info| info.texture().index());

        let metallic = metallic_roughness.metallic_factor();
        let roughness = metallic_roughness.roughness_factor();
        let metallic_tex = metallic_roughness
            .metallic_roughness_texture()
            .map(|info| info.texture().index());

        let normal = mat.normal_texture().map(|norm| norm.texture().index());
        let (occlusion_tex, occlusion_str) = mat
            .occlusion_texture()
            .map(|occ| (Some(occ.texture().index()), occ.strength()))
            .unwrap_or((None, 0.0));

        Self {
            base_color,
            base_tex,
            metallic,
            roughness,
            metallic_tex,
            normal,
            occlusion_tex,
            occlusion_str,
        }
    }
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
