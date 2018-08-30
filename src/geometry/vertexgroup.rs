use std::sync::Arc;

use vulkano::buffer::BufferUsage;

use buffer::CpuAccessibleBufferAutoPool;
use geometry::VertexPositionNormalUVColor;
use renderer::Renderer;


// TODO: linking vertgroup to material by id field is probably fragile
pub struct VertexGroup {
    pub vertices: Vec<VertexPositionNormalUVColor>,
    pub vertex_buffer: Option<Arc<CpuAccessibleBufferAutoPool<[VertexPositionNormalUVColor]>>>,
    pub indices: Vec<u32>,
    pub index_buffer: Option<Arc<CpuAccessibleBufferAutoPool<[u32]>>>,
    pub material_id: u8,
}


impl VertexGroup {
    pub fn new(verts: Vec<VertexPositionNormalUVColor>, idxs: Vec<u32>, mat_id: u8, renderer: &Renderer) -> VertexGroup {
        let mut group = VertexGroup {
            vertices: verts.to_vec(),
            vertex_buffer: None,
            indices: idxs.to_vec(),
            index_buffer: None,
            material_id: mat_id
        };
        group.update_buffers(renderer);
        group
    }


    pub fn update_buffers(&mut self, renderer: &Renderer) {
        self.update_vertex_buffer(renderer);
        self.update_index_buffer(renderer);
    }


    pub fn update_vertex_buffer(&mut self, renderer: &Renderer) {
        self.vertex_buffer = Some(CpuAccessibleBufferAutoPool::from_iter(renderer.device.clone(), renderer.memory_pool.clone(), BufferUsage::all(), self.vertices.iter().cloned()).expect("failed to create vertex buffer"));
    }


    pub fn update_index_buffer(&mut self, renderer: &Renderer) {
        self.index_buffer = Some(CpuAccessibleBufferAutoPool::from_iter(renderer.device.clone(), renderer.memory_pool.clone(), BufferUsage::all(), self.indices.iter().cloned()).expect("failed to create index buffer"));
    }
}