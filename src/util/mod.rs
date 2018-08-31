mod aabb;
pub use self::aabb::AABB;

use cgmath::{Vector3, Point3, Quaternion, Deg, Matrix4, EuclideanSpace};


#[derive(Clone)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: f32
}


#[allow(dead_code)]
impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_sv(0.0, Vector3::new(0.0, 0.0, 0.0)),
            scale: 1.0
        }
    }

    pub fn from_position(position: Point3<f32>) -> Transform {
        Transform {
            position,
            rotation: Quaternion::from_sv(0.0, Vector3::new(0.0, 0.0, 0.0)),
            scale: 1.0
        }
    }

    pub fn from_rotation(rotation: Quaternion<f32>) -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation,
            scale: 1.0
        }
    }

    pub fn from_scale(scale: f32) -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_sv(0.0, Vector3::new(0.0, 0.0, 0.0)),
            scale
        }
    }

    pub fn to_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position.to_vec()) * Matrix4::from(self.rotation) * Matrix4::from_scale(self.scale)
    }
}


pub struct Camera {
    pub fov: Deg<f32>
}


impl Camera {
    pub fn new() -> Camera {
        Camera {
            fov: Deg(45.0)
        }
    }
}


pub mod cube {
    use ::geometry::{VertexPositionNormalUVColor, VertexPositionColorAlpha};

    pub fn generate_unit_cube(x: i32, y: i32, z: i32) -> [VertexPositionNormalUVColor; 24] {
        let x = x as f32;
        let y = y as f32;
        let z = z as f32;
        [
            VertexPositionNormalUVColor { position: [ x+1.0, y,     z     ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 0.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x,     y,     z     ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 1.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x,     y+1.0, z     ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 1.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x+1.0, y+1.0, z     ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 0.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },

            VertexPositionNormalUVColor { position: [ x+1.0, y,     z+1.0 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 0.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x+1.0, y,     z     ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 1.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x+1.0, y+1.0, z     ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 1.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x+1.0, y+1.0, z+1.0 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 0.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },

            VertexPositionNormalUVColor { position: [ x,     y,     z+1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 0.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x+1.0, y,     z+1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 1.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x+1.0, y+1.0, z+1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 1.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x,     y+1.0, z+1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 0.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },

            VertexPositionNormalUVColor { position: [ x,     y,     z     ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 0.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x,     y,     z+1.0 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 1.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x,     y+1.0, z+1.0 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 1.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x,     y+1.0, z     ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 0.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },

            VertexPositionNormalUVColor { position: [ x+1.0, y,     z+1.0 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 0.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x,     y,     z+1.0 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 1.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x,     y,     z     ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 1.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x+1.0, y,     z     ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 0.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },

            VertexPositionNormalUVColor { position: [ x,     y+1.0, z+1.0 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 0.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x+1.0, y+1.0, z+1.0 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 1.0, 0.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x+1.0, y+1.0, z     ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 1.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },
            VertexPositionNormalUVColor { position: [ x,     y+1.0, z     ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 0.0, 1.0 ], color: [ 1.0, 1.0, 1.0 ] },
        ]
    }


    /// Generate indices for cube given an offset.
    /// Offset is the number of cubes to offset by, not the number of indices.
    pub fn generate_indices_with_offset(offset: u32) -> [u32; 36] {
        let o = offset * 24;
        [
            0+o,  1+o,  2+o,  2+o,  3+o,  0+o,
            4+o,  5+o,  6+o,  6+o,  7+o,  4+o,
            8+o,  9+o, 10+o, 10+o, 11+o,  8+o,
            12+o, 13+o, 14+o, 14+o, 15+o, 12+o,
            16+o, 17+o, 18+o, 18+o, 19+o, 16+o,
            20+o, 21+o, 22+o, 22+o, 23+o, 20+o
        ]
    }


    pub fn generate_chunk_debug_line_vertices(x: i32, y: i32, z: i32, a: f32) -> [VertexPositionColorAlpha; 8] {
        let x = x as f32 * 16f32;
        let y = y as f32 * 16f32;
        let z = z as f32 * 16f32;
        [
            // top
            VertexPositionColorAlpha { position: [ x,      y+16.0, z+16.0 ], color: [ 1.0, 1.0, 1.0, a ] },
            VertexPositionColorAlpha { position: [ x+16.0, y+16.0, z+16.0 ], color: [ 1.0, 1.0, 1.0, a ] },
            VertexPositionColorAlpha { position: [ x+16.0, y+16.0, z      ], color: [ 1.0, 1.0, 1.0, a ] },
            VertexPositionColorAlpha { position: [ x,      y+16.0, z      ], color: [ 1.0, 1.0, 1.0, a ] },
            // bottom
            VertexPositionColorAlpha { position: [ x,      y, z+16.0 ], color: [ 1.0, 1.0, 1.0, a ] },
            VertexPositionColorAlpha { position: [ x+16.0, y, z+16.0 ], color: [ 1.0, 1.0, 1.0, a ] },
            VertexPositionColorAlpha { position: [ x+16.0, y, z      ], color: [ 1.0, 1.0, 1.0, a ] },
            VertexPositionColorAlpha { position: [ x,      y, z      ], color: [ 1.0, 1.0, 1.0, a ] },
        ]
    }


    pub fn generate_chunk_debug_line_indices(offset: u32) -> [u32; 24] {
        let o = offset * 8;
        [
            0+o,  1+o,  1+o,  2+o,  2+o,  3+o, 3+o, 0+o, // top
            0+o,  4+o,  1+o,  5+o,  2+o,  6+o, 3+o, 7+o, // middle
            4+o,  5+o,  5+o,  6+o,  6+o,  7+o, 7+o, 4+o, // bottom
        ]
    }
}
