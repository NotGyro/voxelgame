use std::sync::Arc;

use geometry::{VertexGroup, Material};
use renderer::ChunkRenderQueueEntry;
use util::Transform;


pub struct Mesh {
    pub transform: Transform,
    pub vertex_groups: Vec<Arc<VertexGroup>>,
    pub materials: Vec<Material>
}


impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            transform: Transform::new(),
            vertex_groups: Vec::new(),
            materials: Vec::new(),
        }
    }


    pub fn queue(&self) -> Vec<ChunkRenderQueueEntry> {
        let mut result = Vec::new();
        for vg in self.vertex_groups.iter() {
            result.push(ChunkRenderQueueEntry {
                vertex_group: vg.clone(),
                material: self.materials[vg.material_id as usize].clone(),
                transform: self.transform.to_matrix()
            });
        }
        result
    }
}