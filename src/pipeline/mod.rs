use std::sync::Arc;

// use cgmath::Matrix4;
use vulkano::image::swapchain::SwapchainImage;
use winit::Window;
use vulkano::format::D32Sfloat;
use vulkano::image::attachment::AttachmentImage;
use vulkano::pipeline::GraphicsPipelineAbstract;
// use vulkano::device::Queue;
// use vulkano::command_buffer::AutoCommandBuffer;

// use registry::TextureRegistry;
// use util::Transform;


pub mod chunk_pipeline;
pub use self::chunk_pipeline::ChunkRenderPipeline;
pub mod lines_pipeline;
pub use self::lines_pipeline::LinesRenderPipeline;


pub trait RenderPipeline {
    fn pipeline(&self) -> &Arc<GraphicsPipelineAbstract + Send + Sync>;
    fn remove_framebuffers(&mut self);
    fn recreate_framebuffers(&mut self, images: &Vec<Arc<SwapchainImage<Window>>>, depth_buffer: &Arc<AttachmentImage<D32Sfloat>>);
    //fn build_command_buffer(&self, image_num: usize, queue: &Arc<Queue>, dimensions: [u32; 2], transform: Transform, view_mat: Matrix4<f32>, proj_mat: Matrix4<f32>, tex_registry: &TextureRegistry) -> AutoCommandBuffer;
}