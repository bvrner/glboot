use crate::ogl::buffers::Layout;
use cgmath::{Vector2, Vector3};

// That's the way I found to create simpler or more complex meshs as needed
// since using a full vertex would consume more memory than needed sometimes
// and it's also easier to support multiple file types this way

#[derive(Debug, Default)]
pub struct RawVertex {
    pub vertices: Vec<Vector3<f32>>,
    pub normals: Vec<Vector3<f32>>,
    pub tex_coords: Vec<Vector2<f32>>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BasicVertex {
    pub vertice: Vector3<f32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TexVertex {
    pub vertice: Vector3<f32>,
    pub tex_coords: Vector2<f32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NormVertex {
    pub vertice: Vector3<f32>,
    pub normal: Vector3<f32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StandardVertex {
    pub vertice: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coords: Vector2<f32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FullVertex {
    pub vertice: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coords: Vector2<f32>,
    pub tangent: Vector3<f32>,
    pub bitangent: Vector3<f32>,
}

pub trait VertexData {
    fn get_layout() -> Layout;
    fn from_raw(data: RawVertex) -> Vec<Self>
    where
        Self: std::marker::Sized;
}

// maybe I could have use a macro to do all that, but oh well

impl VertexData for BasicVertex {
    fn get_layout() -> Layout {
        layout![(3, f32, gl::FLOAT)]
    }
    fn from_raw(data: RawVertex) -> Vec<Self> {
        data.vertices
            .into_iter()
            .map(|p| Self { vertice: p })
            .collect()
    }
}

impl VertexData for TexVertex {
    fn get_layout() -> Layout {
        layout![(3, f32, gl::FLOAT), (2, f32, gl::FLOAT)]
    }
    fn from_raw(data: RawVertex) -> Vec<Self> {
        let tex_iter = if data.normals.len() > 0 {
            data.tex_coords.into_iter().cycle()
        } else {
            vec![Vector2::new(0.0_f32, 0.0)].into_iter().cycle()
        };

        data.vertices
            .into_iter()
            .zip(tex_iter)
            .map(|(p, t)| Self {
                vertice: p,
                tex_coords: t,
            })
            .collect()
    }
}

impl VertexData for NormVertex {
    fn get_layout() -> Layout {
        layout![(3, f32, gl::FLOAT), (3, f32, gl::FLOAT)]
    }
    fn from_raw(data: RawVertex) -> Vec<Self> {
        let normal_iter = if data.normals.len() > 0 {
            data.normals.into_iter().cycle()
        } else {
            vec![Vector3::new(0.0_f32, 0.0, 0.0)].into_iter().cycle()
        };

        data.vertices
            .into_iter()
            .zip(normal_iter)
            .map(|(p, n)| Self {
                vertice: p,
                normal: n,
            })
            .collect()
    }
}

impl VertexData for StandardVertex {
    fn get_layout() -> Layout {
        layout![
            (3, f32, gl::FLOAT),
            (3, f32, gl::FLOAT),
            (2, f32, gl::FLOAT)
        ]
    }
    fn from_raw(data: RawVertex) -> Vec<Self> {
        // the way I found to handle possible missing normals and tex coords
        let normal_iter = if data.normals.len() > 0 {
            data.normals.into_iter().cycle()
        } else {
            vec![Vector3::new(0.0_f32, 0.0, 0.0)].into_iter().cycle()
        };
        let tex_iter = if data.tex_coords.len() > 0 {
            data.tex_coords.into_iter().cycle()
        } else {
            vec![Vector2::new(0.0_f32, 0.0)].into_iter().cycle()
        };

        data.vertices
            .into_iter()
            .zip(normal_iter)
            .zip(tex_iter)
            .map(|((p, n), t)| Self {
                vertice: p,
                normal: n,
                tex_coords: t,
            })
            .collect()
    }
}

impl VertexData for FullVertex {
    fn get_layout() -> Layout {
        layout![
            (3, f32, gl::FLOAT),
            (3, f32, gl::FLOAT),
            (2, f32, gl::FLOAT),
            (3, f32, gl::FLOAT),
            (3, f32, gl::FLOAT)
        ]
    }

    fn from_raw(data: RawVertex) -> Vec<Self> {
        // same as above
        let normal_iter = if data.normals.len() > 0 {
            data.normals.into_iter().cycle()
        } else {
            vec![Vector3::new(0.0_f32, 0.0, 0.0)].into_iter().cycle()
        };
        let tex_iter = if data.tex_coords.len() > 0 {
            data.tex_coords.into_iter().cycle()
        } else {
            vec![Vector2::new(0.0_f32, 0.0)].into_iter().cycle()
        };

        data.vertices
            .into_iter()
            .zip(normal_iter)
            .zip(tex_iter)
            .map(|((p, n), t)| Self {
                vertice: p,
                normal: n,
                tex_coords: t,
                tangent: Vector3::new(0.0, 0.0, 0.0),
                bitangent: Vector3::new(0.0, 0.0, 0.0),
            })
            .collect()
    }
    // Compute the tangent and bitanget for each triangle on the mesh
    // I'll just trust no indice goes out of bounds
    // for triangle in self.indices.chunks_exact_mut(3) {
    //     let v0 = &self.vertices[triangle[0] as usize].vertice;
    //     let v1 = &self.vertices[triangle[1] as usize].vertice;
    //     let v2 = &self.vertices[triangle[2] as usize].vertice;

    //     let uv0 = &self.vertices[triangle[0] as usize].tex_coords;
    //     let uv1 = &self.vertices[triangle[1] as usize].tex_coords;
    //     let uv2 = &self.vertices[triangle[2] as usize].tex_coords;

    //     let delta1 = v1 - v0;
    //     let delta2 = v2 - v0;

    //     let deltau1 = uv1 - uv0;
    //     let deltau2 = uv2 - uv0;

    //     let r = 1.0 / (deltau1.x * deltau2.y - deltau1.y * deltau2.x);

    //     let tangent = (delta1 * deltau2.y - delta2 * deltau1.y) * r;
    //     let bitangent = (delta2 * deltau1.x - delta1 * deltau2.x) * r;

    //     self.vertices[triangle[0] as usize].tangent = tangent;
    //     self.vertices[triangle[1] as usize].tangent = tangent;
    //     self.vertices[triangle[2] as usize].tangent = tangent;

    //     self.vertices[triangle[0] as usize].bitangent = bitangent;
    //     self.vertices[triangle[1] as usize].bitangent = bitangent;
    //     self.vertices[triangle[2] as usize].bitangent = bitangent;
    // }
}
