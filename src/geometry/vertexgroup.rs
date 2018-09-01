//! A vertex group type, which holds vertex and index buffers and a material id.
//!
//! Material id is a `u8` which corresponds to the index of a material in the owning [Mesh](super::Mesh).

use std::sync::Arc;

use vulkano::buffer::BufferUsage;

use buffer::CpuAccessibleBufferAutoPool;
use geometry::VertexPositionNormalUVColor;
use renderer::Renderer;


// TODO: linking vertgroup to material by id field is probably fragile
// TODO: storing vertex data as a Vec *and* in the buffer is probably unnecessary.
/// Vertex group object. Material id is a `u8` which corresponds to the index of a material in the owning [Mesh](super::Mesh).
///
/// See [module-level documentation](self).
pub struct VertexGroup {
    /// Vertex data. Set this and call [update_vertex_buffer](VertexGroup::update_vertex_buffer) to update the buffer.
    pub vertices: Vec<VertexPositionNormalUVColor>,
    /// Vertex buffer. Cpu-accessible, managed by [AutoMemoryPool](::pool::AutoMemoryPool).
    pub vertex_buffer: Option<Arc<CpuAccessibleBufferAutoPool<[VertexPositionNormalUVColor]>>>,
    /// Index data. Set this and call [update_index_buffer](VertexGroup::update_index_buffer) to update the buffer.
    pub indices: Vec<u32>,
    /// Index buffer. Cpu-accessible, managed by [AutoMemoryPool](::pool::AutoMemoryPool).
    pub index_buffer: Option<Arc<CpuAccessibleBufferAutoPool<[u32]>>>,
    /// Corresponds to the index of a material in the owning [Mesh](super::Mesh).
    pub material_id: u8,
}


impl VertexGroup {
    /// Constructs a new `VertexGroup` with the given parameters.
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


    /// Updates both buffers with data from their respective `Vec`s.
    pub fn update_buffers(&mut self, renderer: &Renderer) {
        self.update_vertex_buffer(renderer);
        self.update_index_buffer(renderer);
    }


    /// Updates the vertex buffer with data from `vertex_buffer`.
    pub fn update_vertex_buffer(&mut self, renderer: &Renderer) {
        self.vertex_buffer = Some(CpuAccessibleBufferAutoPool::from_iter(renderer.device.clone(), renderer.memory_pool.clone(), BufferUsage::all(), self.vertices.iter().cloned()).expect("failed to create vertex buffer"));
    }


    /// Updates the index buffer with data from `index_buffer`.
    pub fn update_index_buffer(&mut self, renderer: &Renderer) {
        self.index_buffer = Some(CpuAccessibleBufferAutoPool::from_iter(renderer.device.clone(), renderer.memory_pool.clone(), BufferUsage::all(), self.indices.iter().cloned()).expect("failed to create index buffer"));
    }
}