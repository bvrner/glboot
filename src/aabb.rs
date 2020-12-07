use cgmath::{vec3, Point3, Vector3};
use gltf::mesh::BoundingBox;

// maybe use a lazy static shader instead?
pub const SOURCE_V: &str = "#version 330 core\n layout (location = 0) in vec3 aPos;
uniform mat4 trans; uniform mat4 view; uniform mat4 proj; void main() { gl_Position = proj * view *  trans * vec4(aPos, 1.0); }";
pub const SOURCE_F: &str =
    "#version 330 core\n out vec4 Col; void main() { Col = vec4(1.0, 1.0, 0.0, 1.0); }";

#[derive(Debug, Copy, Clone)]
pub struct Aabb {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl Aabb {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    pub fn surrounds(&self, other: &Self) -> Self {
        let min = Point3::new(
            self.min.x.min(other.min.x),
            self.min.y.min(other.min.y),
            self.min.z.min(other.min.z),
        );
        let max = Point3::new(
            self.max.x.max(other.max.x),
            self.max.y.max(other.max.y),
            self.max.z.max(other.max.z),
        );

        Self { min, max }
    }

    pub fn gen_vertices(&self) -> (Vec<Vector3<f32>>, Vec<u32>) {
        let v = vec![
            // front
            vec3(self.max.x, self.max.y, self.min.z), // top right
            vec3(self.max.x, self.min.y, self.min.z), // bottom right
            vec3(self.min.x, self.max.y, self.min.z), // top left
            vec3(self.min.x, self.min.y, self.min.z), // bottom left
            // top
            vec3(self.max.x, self.max.y, self.max.z), // top right
            vec3(self.max.x, self.max.y, self.min.z), // bottom right
            vec3(self.min.x, self.max.y, self.max.z), // top left
            vec3(self.min.x, self.max.y, self.min.z), // bottom left
            // bottom
            vec3(self.max.x, self.min.y, self.max.z), // top right
            vec3(self.max.x, self.min.y, self.min.z), // bottom right
            vec3(self.min.x, self.min.y, self.max.z), // top left
            vec3(self.min.x, self.min.y, self.min.z), // bottom left
            // back
            vec3(self.max.x, self.max.y, self.max.z), // top right
            vec3(self.max.x, self.min.y, self.max.z), // bottom right
            vec3(self.min.x, self.max.y, self.max.z), // top left
            vec3(self.min.x, self.min.y, self.max.z), // bottom left
            // left
            vec3(self.min.x, self.max.y, self.min.z), // top right
            vec3(self.min.x, self.min.y, self.min.z), // bottom right
            vec3(self.min.x, self.max.y, self.max.z), // top left
            vec3(self.min.x, self.min.y, self.max.z), // bottom left
            // right
            vec3(self.max.x, self.max.y, self.max.z), // top right
            vec3(self.max.x, self.min.y, self.max.z), // bottom right
            vec3(self.max.x, self.max.y, self.min.z), // top left
            vec3(self.max.x, self.min.y, self.min.z), // bottom left
        ];

        let i = vec![
            // front
            0,
            1,
            2,
            1,
            2,
            3,
            // top
            0 + 4,
            1 + 4,
            2 + 4,
            1 + 4,
            2 + 4,
            3 + 4,
            // bottom
            0 + 8,
            1 + 8,
            2 + 8,
            1 + 8,
            2 + 8,
            3 + 8,
            // back
            0 + 12,
            1 + 12,
            2 + 12,
            1 + 12,
            2 + 12,
            3 + 12,
            // left
            0 + 16,
            1 + 16,
            2 + 16,
            1 + 16,
            2 + 16,
            3 + 16,
            // right
            0 + 20,
            1 + 20,
            2 + 20,
            1 + 20,
            2 + 20,
            3 + 20,
        ];

        (v, i)
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
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
