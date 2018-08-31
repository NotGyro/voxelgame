pub mod chunk_pipeline;
pub mod lines_pipeline;
pub mod skybox_pipeline;
pub use self::chunk_pipeline::ChunkRenderPipeline;
pub use self::lines_pipeline::LinesRenderPipeline;
pub use self::skybox_pipeline::SkyboxRenderPipeline;


// TODO: make render pipelines generic
/*use std::sync::Arc;

use vulkano::device::Device;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::buffer::cpu_pool::CpuBufferPool;


pub struct RenderPipelineData<T> {
    pub device: Arc<Device>,
    pub vulkan_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    pub framebuffers: Option<Vec<Arc<FramebufferAbstract + Send + Sync>>>,
    pub renderpass: Arc<RenderPassAbstract + Send + Sync>,
    pub uniform_buffer_pool: CpuBufferPool<T>,
}*/