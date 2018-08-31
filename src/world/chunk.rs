use std::sync::Arc;

use cgmath::Point3;

use ::geometry::{Mesh, VertexPositionNormalUVColor, VertexGroup, Material};
use ::util::Transform;
use ::renderer::Renderer;
use mesh_simplifier::{MeshSimplifier, QuadFacing};


/// Struct representing blocks in a 16x16x16 chunk.
///
/// Encoded in axis order, X, Y, Z. (Z coords are consecutive for a given Y coord, etc).
pub struct Chunk {
    pub ids: [u8; 16*16*16],
    pub position: (i32, i32, i32),
    pub dimension_id: u32,
    pub mesh: Mesh,
    pub mesh_dirty: bool
}


impl Chunk {
    /// Constructs a new (empty) chunk.
    pub fn new(position: (i32, i32, i32), dimension_id: u32) -> Chunk {
        Chunk {
            ids: [0; 16*16*16],
            position,
            dimension_id,
            mesh: Mesh::new(),
            mesh_dirty: false
        }
    }


    /// Converts a flat index to (x, y, z) coordinates.
    #[allow(dead_code)]
    pub fn i_to_xyz(i: usize) -> (i32, i32, i32) { (i as i32/(16*16), (i as i32/16) % 16, i as i32 % 16) }


    /// Converts (x, y, z) coordinates to a flat index.
    #[allow(dead_code)]
    pub fn xyz_to_i(x: i32, y: i32, z: i32) -> usize { ((x * 16*16) + (y * 16) + z) as usize }


    #[allow(dead_code)]
    /// Sets a block at the given index.
    pub fn set_at(&mut self, i: usize, id: u8) {
        self.ids[i] = id;
    }


    pub fn replace_data(&mut self, data: &[u8; 16*16*16]) {
        self.ids = *data;
    }


    pub fn generate_mesh(&mut self, renderer: &Renderer) {
        let quad_lists = MeshSimplifier::generate_mesh(self);
        let mut mesh = Mesh::new();
        let mut vertices = Vec::new() as Vec<VertexPositionNormalUVColor>;
        let mut indices = Vec::new() as Vec<u32>;
        let mut o = 0;
        for (facing, layer, list) in quad_lists.iter() {
            for quad in list {
                match facing {
                    QuadFacing::Left => {
                        vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32, quad.x as f32,          (quad.y+quad.h) as f32 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32, (quad.x+quad.w) as f32, (quad.y+quad.h) as f32 ], normal: [ -1.0, 0.0, 0.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32, (quad.x+quad.w) as f32, quad.y as f32          ], normal: [ -1.0, 0.0, 0.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32, quad.x as f32,          quad.y as f32          ], normal: [ -1.0, 0.0, 0.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                    },
                    QuadFacing::Right => {
                        vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32 + 1.0, (quad.x+quad.w) as f32, (quad.y+quad.h) as f32 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32 + 1.0, quad.x as f32,          (quad.y+quad.h) as f32 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32 + 1.0, quad.x as f32,          quad.y as f32          ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32 + 1.0, (quad.x+quad.w) as f32, quad.y as f32          ], normal: [ 1.0, 0.0, 0.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                    },
                    QuadFacing::Bottom => {
                        vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, *layer as f32, (quad.y+quad.h) as f32 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          *layer as f32, (quad.y+quad.h) as f32 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          *layer as f32, quad.y as f32          ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, *layer as f32, quad.y as f32          ], normal: [ 0.0, -1.0, 0.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                    },
                    QuadFacing::Top => {
                        vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          *layer as f32 + 1.0, (quad.y+quad.h) as f32 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, *layer as f32 + 1.0, (quad.y+quad.h) as f32 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, *layer as f32 + 1.0, quad.y as f32          ], normal: [ 0.0, 1.0, 0.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          *layer as f32 + 1.0, quad.y as f32          ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                    },
                    QuadFacing::Front => {
                        vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          (quad.y+quad.h) as f32, *layer as f32 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, (quad.y+quad.h) as f32, *layer as f32 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, quad.y as f32,          *layer as f32 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          quad.y as f32,          *layer as f32 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                    },
                    QuadFacing::Back => {
                        vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, (quad.y+quad.h) as f32, *layer as f32 + 1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          (quad.y+quad.h) as f32, *layer as f32 + 1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          quad.y as f32,          *layer as f32 + 1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, quad.y as f32,          *layer as f32 + 1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                    },
                }
                indices.push(0+o); indices.push(1+o); indices.push(2+o);
                indices.push(2+o); indices.push(3+o); indices.push(0+o);
                o += 4;
            }
        }
        mesh.vertex_groups.push(Arc::new(VertexGroup::new(vertices, indices, 0, renderer)));
        mesh.materials.push(Material { albedo_map_name: String::from("dirt") });
        mesh.transform = Transform::from_position(Point3::new(self.position.0 as f32 * 16.0,
                                                              self.position.1 as f32 * 16.0,
                                                              self.position.2 as f32 * 16.0));
        self.mesh = mesh;
        self.mesh_dirty = false;

        // TODO: reimplement multiple materials per chunk
//        // get all unique ids
//        let mut unique_ids = HashSet::new() as HashSet<u8>;
//        for i in 0..(16*16*16) {
//            unique_ids.insert(self.ids[i]);
//        }
//        unique_ids.remove(&0u8); // don't generate anything for air
//
//        let mut mesh = Mesh::new();
//
//        for id in unique_ids.iter() {
//            let mut vertices = Vec::new() as Vec<VertexPositionNormalUVColor>;
//            vertices.reserve(24 * 16 * 16 * 16);
//            let mut indices = Vec::new() as Vec<u32>;
//            indices.reserve(8 * 16 * 16 * 16);
//            let mut index_offset = 0;
//
//            for x in 0..16 {
//                for y in 0..16 {
//                    for z in 0..16 {
//                        if self.ids[Chunk::xyz_to_i(x, y, z)] == *id {
//                            let mut verts = ::util::cube::generate_unit_cube(x, y, z).to_vec();
//                            vertices.append(&mut verts);
//                            indices.append(&mut ::util::cube::generate_indices_with_offset(index_offset).to_vec());
//                            index_offset += 1;
//                        }
//                    }
//                }
//            }
//
//            mesh.vertex_groups.push(Arc::new(VertexGroup::new(vertices, indices, *id, renderer)));
//        }
//        mesh.materials.push(Material { albedo_map_name: String::from("") });
//        mesh.materials.push(Material { albedo_map_name: String::from("stone") });
//        mesh.materials.push(Material { albedo_map_name: String::from("dirt") });
//        mesh.materials.push(Material { albedo_map_name: String::from("grass") });
//
//        mesh.transform = Transform::from_position(Point3::new(self.position.0 as f32 * 16.0,
//                                                              self.position.1 as f32 * 16.0,
//                                                              self.position.2 as f32 * 16.0));
//
//        self.mesh = mesh;
//        self.mesh_dirty = false;
    }
}