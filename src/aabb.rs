use cgmath::{vec3, Matrix4, Vector3};
use gltf::mesh::BoundingBox;

// maybe use a lazy static shader instead?
pub const SOURCE_V: &str = "#version 330 core\n layout (location = 0) in vec3 aPos;
uniform mat4 trans; uniform mat4 view; uniform mat4 proj; void main() { gl_Position = proj * view *  trans * vec4(aPos, 1.0); }";
pub const SOURCE_F: &str =
    "#version 330 core\n out vec4 Col; void main() { Col = vec4(1.0, 1.0, 0.0, 1.0); }";

#[derive(Debug, Copy, Clone)]
pub struct Aabb {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Aabb {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> Self {
        Self { min, max }
    }

    pub fn surrounds(&self, other: &Self) -> Self {
        let min = Vector3::new(
            self.min.x.min(other.min.x),
            self.min.y.min(other.min.y),
            self.min.z.min(other.min.z),
        );
        let max = Vector3::new(
            self.max.x.max(other.max.x),
            self.max.y.max(other.max.y),
            self.max.z.max(other.max.z),
        );

        Self { min, max }
    }

    #[inline]
    pub fn transform(&self, mat: &Matrix4<f32>) -> Self {
        Self {
            min: (mat * self.min.extend(1.0)).truncate(),
            max: (mat * self.max.extend(1.0)).truncate(),
        }
    }

    pub fn gen_vertices(&self) -> (Vec<Vector3<f32>>, Vec<u32>) {
        let v = vec![
            vec3(self.min.x, self.min.y, self.max.z),
            vec3(self.max.x, self.min.y, self.max.z),
            vec3(self.max.x, self.max.y, self.max.z),
            vec3(self.min.x, self.max.y, self.max.z),
            vec3(self.min.x, self.min.y, self.min.z),
            vec3(self.max.x, self.min.y, self.min.z),
            vec3(self.max.x, self.max.y, self.min.z),
            vec3(self.min.x, self.max.y, self.min.z),
        ];
        let i = vec![0, 1, 2, 3, 4, 5, 6, 7, 0, 4, 1, 5, 2, 6, 3, 7];

        (v, i)
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }
}

impl From<BoundingBox> for Aabb {
    fn from(b: BoundingBox) -> Self {
        Self {
            min: b.min.into(),
            max: b.max.into(),
        }
    }
}
