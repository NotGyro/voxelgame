use std::sync::Arc;

use cgmath::Matrix4;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::command_buffer::{AutoCommandBufferBuilder, AutoCommandBuffer, DynamicState};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, Queue};
use vulkano::format::D32Sfloat;
use vulkano::framebuffer::{FramebufferAbstract, Framebuffer, RenderPass, RenderPassDesc, Subpass};
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain::Swapchain;
use winit::Window;

use buffer::CpuAccessibleBufferAutoPool;
use geometry::VertexPositionColorAlpha;
use pool::AutoMemoryPool;
use renderpass::RenderPassUnclearedColorWithDepth;
use shader::lines as LinesShaders;


// temp struct for debug drawing lines
struct LineData {
    pub vertex_buffer: Arc<CpuAccessibleBufferAutoPool<[VertexPositionColorAlpha]>>,
    pub index_buffer: Arc<CpuAccessibleBufferAutoPool<[u32]>>,
}


pub struct LinesRenderPipeline {
    device: Arc<Device>,
    vulkan_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    pub framebuffers: Option<Vec<Arc<FramebufferAbstract + Send + Sync>>>,
    renderpass: Arc<RenderPass<RenderPassUnclearedColorWithDepth>>,
    uniform_buffer_pool: CpuBufferPool<LinesShaders::vertex::ty::Data>,
    temp_line_data: LineData,
}


impl LinesRenderPipeline {
    pub fn new(swapchain: &Swapchain<Window>, device: &Arc<Device>, memory_pool: &AutoMemoryPool) -> LinesRenderPipeline {
        let vs = LinesShaders::vertex::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = LinesShaders::fragment::Shader::load(device.clone()).expect("failed to create shader module");

        let renderpass= Arc::new(
            RenderPassUnclearedColorWithDepth { color_format: swapchain.format() }
                .build_render_pass(device.clone())
                .unwrap()
        );

        let pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<VertexPositionColorAlpha>()
            .vertex_shader(vs.main_entry_point(), ())
            .line_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .blend_alpha_blending()
            .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap());

        let mut temp_line_verts = Vec::new();
        let mut temp_line_idxs = Vec::new();
        let mut line_idx_offset = 0;
        for x in 0..8 {
            for z in 0..8 {
                temp_line_verts.append(&mut ::util::cube::generate_chunk_debug_line_vertices(x, 0, z, 0.25f32).to_vec());
                temp_line_idxs.append(&mut ::util::cube::generate_chunk_debug_line_indices(line_idx_offset).to_vec());
                line_idx_offset += 1;
            }
        }

        let temp_line_data = LineData {
            vertex_buffer: CpuAccessibleBufferAutoPool::<[VertexPositionColorAlpha]>::from_iter(device.clone(), memory_pool.clone(), BufferUsage::all(), temp_line_verts.iter().cloned()).expect("failed to create buffer"),
            index_buffer: CpuAccessibleBufferAutoPool::<[u32]>::from_iter(device.clone(), memory_pool.clone(), BufferUsage::all(), temp_line_idxs.iter().cloned()).expect("failed to create buffer"),
        };

        LinesRenderPipeline {
            device: device.clone(),
            vulkan_pipeline: pipeline,
            framebuffers: None,
            renderpass,
            uniform_buffer_pool: CpuBufferPool::<LinesShaders::vertex::ty::Data>::new(device.clone(), BufferUsage::all()),
            temp_line_data,
        }
    }


    pub fn build_command_buffer(&self, image_num: usize, queue: &Arc<Queue>, dimensions: [u32; 2], view_mat: Matrix4<f32>, proj_mat: Matrix4<f32>) -> AutoCommandBuffer {
        let descriptor_set;
        let subbuffer = self.uniform_buffer_pool.next(LinesShaders::vertex::ty::Data {
            world: Matrix4::from_scale(1.0).into(),
            view: view_mat.into(),
            proj: proj_mat.into(),
        }).unwrap();
        descriptor_set = Arc::new(PersistentDescriptorSet::start(self.vulkan_pipeline.clone(), 0)
            .add_buffer(subbuffer).unwrap()
            .build().unwrap()
        );

        AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), queue.family())
            .unwrap()
            .begin_render_pass(
                self.framebuffers.as_ref().unwrap()[image_num].clone(), false,
                vec![::vulkano::format::ClearValue::None, ::vulkano::format::ClearValue::None]).unwrap()
            .draw_indexed(self.vulkan_pipeline.clone(), &DynamicState {
                line_width: None,
                viewports: Some(vec![Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }]),
                scissors: None,
            },
                          vec![self.temp_line_data.vertex_buffer.clone()],
                          self.temp_line_data.index_buffer.clone(),
                          descriptor_set.clone(), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap()
    }


    pub fn remove_framebuffers(&mut self) { self.framebuffers = None; }


    pub fn recreate_framebuffers(&mut self, images: &Vec<Arc<SwapchainImage<Window>>>, depth_buffer: &Arc<AttachmentImage<D32Sfloat>>) {
        let new_framebuffers = Some(images.iter().map(|image| {
            let arc: Arc<FramebufferAbstract + Send + Sync> = Arc::new(Framebuffer::start(self.renderpass.clone())
                .add(image.clone()).unwrap()
                .add(depth_buffer.clone()).unwrap()
                .build().unwrap());
            arc
        }).collect::<Vec<_>>());
        ::std::mem::replace(&mut self.framebuffers, new_framebuffers);
    }
}