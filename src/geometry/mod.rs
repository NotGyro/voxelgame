pub mod mesh;
pub mod vertex;
pub mod vertexgroup;

pub use self::mesh::Mesh;
pub use self::vertex::{VertexPositionNormalUVColor, VertexPositionColorAlpha};
pub use self::vertexgroup::VertexGroup;


#[derive(Clone)]
pub struct Material {
    pub albedo_map_name: String
}
