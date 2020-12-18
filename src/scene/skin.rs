use cgmath::Matrix4;
use gltf::Skin as gltfSkin;

#[derive(Debug, Clone)]
pub struct Skin {
    pub joints: Vec<Joint>,
}

impl Skin {
    pub fn from_gltf(skin: &gltfSkin, buf: &[gltf::buffer::Data]) -> Self {
        let reader = skin.reader(|buffer| Some(&buf[buffer.index()]));

        let matrices: Vec<Matrix4<f32>> = reader
            .read_inverse_bind_matrices()
            .expect("Skinned mesh should have bind matrices.")
            .map(Matrix4::from)
            .collect();
        let joints: Vec<usize> = skin.joints().map(|node| node.index()).collect();

        Self {
            joints: matrices
                .into_iter()
                .zip(joints.into_iter())
                .map(|(m, n)| Joint::new(m, n))
                .collect(),
        }
    }
}
#[derive(Debug, Copy, Clone)]
pub struct Joint {
    pub bind_matrix: Matrix4<f32>,
    pub node: usize,
}

impl Joint {
    fn new(bind_matrix: Matrix4<f32>, node: usize) -> Self {
        Self { bind_matrix, node }
    }
}
