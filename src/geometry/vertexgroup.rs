use std::sync::Arc;

use vulkano::device::Device;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::buffer::BufferUsage;

use ::geometry::VertexPositionNormalUVColor;


// TODO: linking vertgroup to material by id field is probably fragile
pub struct VertexGroup {
    pub vertices: Vec<VertexPositionNormalUVColor>,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[VertexPositionNormalUVColor]>>>,
    pub indices: Vec<u32>,
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u32]>>>,
    pub material_id: u8,
}


impl VertexGroup {
    pub fn new(verts: Vec<VertexPositionNormalUVColor>, idxs: Vec<u32>, mat_id: u8, device: Arc<Device>) -> VertexGroup {
        let mut group = VertexGroup {
            vertices: verts.to_vec(),
            vertex_buffer: None,
            indices: idxs.to_vec(),
            index_buffer: None,
            material_id: mat_id
        };
        group.update_buffers(device.clone());
        group
    }


    pub fn update_buffers(&mut self, device: Arc<Device>) {
        self.update_vertex_buffer(device.clone());
        self.update_index_buffer(device.clone());
    }


    pub fn update_vertex_buffer(&mut self, device: Arc<Device>) {
        self.vertex_buffer = Some(CpuAccessibleBuffer::<[VertexPositionNormalUVColor]>::from_iter(device.clone(), BufferUsage::all(), self.vertices.iter().cloned()).expect("failed to create buffer"));
    }


    pub fn update_index_buffer(&mut self, device: Arc<Device>) {
        self.index_buffer = Some(CpuAccessibleBuffer::<[u32]>::from_iter(device.clone(), BufferUsage::all(), self.indices.iter().cloned()).expect("failed to create buffer"));
    }
}