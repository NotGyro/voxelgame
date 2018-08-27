use std::sync::Arc;
use std::collections::HashSet;

use vulkano::device::Device;
use cgmath::Point3;

use ::geometry::{Mesh, VertexPositionNormalUVColor, VertexGroup, Material};
use ::util::Transform;


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


    /// Sets a block at the given index.
    pub fn set_at(&mut self, i: usize, id: u8) {
        self.ids[i] = id;
    }


    pub fn generate_mesh(&mut self, device: Arc<Device>) {
        // get all unique ids
        let mut unique_ids = HashSet::new() as HashSet<u8>;
        for i in 0..(16*16*16) {
            unique_ids.insert(self.ids[i]);
        }
        unique_ids.remove(&0u8); // don't generate anything for air

        let mut mesh = Mesh::new();

        for id in unique_ids.iter().clone() {
            let mut vertices = Vec::new() as Vec<VertexPositionNormalUVColor>;
            let mut indices = Vec::new() as Vec<u32>;
            let mut index_offset = 0;

            for i in 0..(16*16*16) {
                if self.ids[i] == *id {
                    let (x, y, z) = Chunk::i_to_xyz(i);
                    let mut verts = ::util::cube::generate_unit_cube(x, y, z).to_vec();
                    vertices.append(&mut verts);
                    indices.append(&mut ::util::cube::generate_indices_with_offset(index_offset).to_vec());
                    index_offset += 1;
                }
            }

            mesh.vertex_groups.push(Arc::new(VertexGroup::new(vertices, indices, *id, device.clone())));
        }
        mesh.materials.push(Material { albedo_map_name: String::from("") });
        mesh.materials.push(Material { albedo_map_name: String::from("stone") });
        mesh.materials.push(Material { albedo_map_name: String::from("dirt") });
        mesh.materials.push(Material { albedo_map_name: String::from("grass") });

        mesh.transform = Transform::from_position(Point3::new(self.position.0 as f32 * 16.0,
                                                              self.position.1 as f32 * 16.0,
                                                              self.position.2 as f32 * 16.0));

        self.mesh = mesh;
        self.mesh_dirty = false;
    }
}