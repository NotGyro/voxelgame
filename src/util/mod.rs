//! Various utility types.


mod aabb;
pub mod logger;
pub use self::aabb::AABB;

use cgmath::{Vector3, Point3, Quaternion, Deg, Matrix4, EuclideanSpace};


/// A 3D transform, with position, rotation, and scale.
#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>
}

#[allow(dead_code)]
impl Transform {
    /// Creates an identity transform.
    pub fn new() -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_sv(0.0, Vector3::new(0.0, 0.0, 0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    /// Creates a transform with the given position.
    pub fn from_position(position: Point3<f32>) -> Transform {
        Transform {
            position,
            rotation: Quaternion::from_sv(0.0, Vector3::new(0.0, 0.0, 0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    /// Creates a transform with the given rotation.
    pub fn from_rotation(rotation: Quaternion<f32>) -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation,
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    /// Creates a transform with the given scale.
    pub fn from_scale(scale: Vector3<f32>) -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_sv(0.0, Vector3::new(0.0, 0.0, 0.0)),
            scale
        }
    }

    /// Creates a transform with the given uniform scale (scaling all axes by the same amount).
    pub fn from_uniform_scale(scale: f32) -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_sv(0.0, Vector3::new(0.0, 0.0, 0.0)),
            scale: Vector3::new(scale, scale, scale),
        }
    }

    /// Generates a 4x4 transformation matrix from this transform.
    pub fn to_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position.to_vec())
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}


pub struct Camera {
    /// Field of fiew.
    pub fov: Deg<f32>
}


impl Camera {
    /// Creates a new Camera.
    pub fn new() -> Camera {
        Camera {
            fov: Deg(45.0)
        }
    }
}


pub mod cube {
    use ::geometry::VertexPositionColorAlpha;


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
