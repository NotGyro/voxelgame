use std::sync::Arc;

use cgmath::{EuclideanSpace, Matrix4, Vector4};

use buffer::CpuAccessibleBufferAutoPool;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::{BufferUsage};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::D32Sfloat;
use vulkano::framebuffer::{FramebufferAbstract, Framebuffer, Subpass, RenderPass, RenderPassDesc};
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Sampler, Filter, SamplerAddressMode, MipmapMode};
use vulkano::swapchain::{Swapchain, Surface, SwapchainCreationError};
use vulkano::sync::{GpuFuture};
use winit::Window;

use util::{Camera, Transform};
use geometry::{VertexPositionNormalUVColor, VertexPositionColorAlpha, VertexGroup, Material};
use renderpass::{RenderPassClearedColorWithDepth, RenderPassUnclearedColorWithDepth};
use registry::TextureRegistry;
use shader::default as DefaultShaders;
use shader::lines as LinesShaders;
use pool::AutoMemoryPool;


static VULKAN_CORRECT_CLIP: Matrix4<f32> = Matrix4 {
    x: Vector4 { x: 1.0, y:  0.0, z: 0.0, w: 0.0 },
    y: Vector4 { x: 0.0, y: -1.0, z: 0.0, w: 0.0 },
    z: Vector4 { x: 0.0, y:  0.0, z: 0.5, w: 0.5 },
    w: Vector4 { x: 0.0, y:  0.0, z: 0.0, w: 1.0 }
};


// temp struct for debug drawing lines
struct LineData {
    pub vertex_buffer: Arc<CpuAccessibleBufferAutoPool<[VertexPositionColorAlpha]>>,
    pub index_buffer: Arc<CpuAccessibleBufferAutoPool<[u32]>>,
}


pub struct RenderQueueMeshEntry {
    pub vertex_group: Arc<VertexGroup>,
    pub material: Material,
    pub transform: Matrix4<f32>
}


pub struct Renderer {
    pub device: Arc<Device>,
    pub memory_pool: AutoMemoryPool,
    queue: Arc<Queue>,
    surface: Arc<Surface<Window>>,
    renderpass: Arc<RenderPass<RenderPassClearedColorWithDepth>>,
    renderpass_lines: Arc<RenderPass<RenderPassUnclearedColorWithDepth>>,
    framebuffers: Option<Vec<Arc<FramebufferAbstract + Send + Sync>>>,
    framebuffers_lines: Option<Vec<Arc<FramebufferAbstract + Send + Sync>>>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    pipeline_lines: Arc<GraphicsPipelineAbstract + Send + Sync>,
    sampler: Arc<Sampler>,
    depth_buffer: Arc<AttachmentImage<D32Sfloat>>,
    uniform_buffer_pool: CpuBufferPool<DefaultShaders::vertex::ty::Data>,
    uniform_buffer_pool_lines: CpuBufferPool<LinesShaders::vertex::ty::Data>,
    recreate_swapchain: bool,
    tex_registry: TextureRegistry,
    temp_line_data: LineData
}


impl Renderer {
    pub fn new(instance: Arc<Instance>, surface: Arc<Surface<Window>>) -> Renderer {
        let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
        let queue = physical.queue_families().find(|&q| q.supports_graphics() &&
            surface.is_supported(q).unwrap_or(false))
            .expect("couldn't find a graphical queue family");

        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            .. DeviceExtensions::none()
        };

        let (device, mut queues) = Device::new(physical, physical.supported_features(),
                                               &device_ext,
                                               [(queue, 0.5)].iter().cloned())
            .expect("failed to create device");
        let queue = queues.next().unwrap();

        let dimensions;
        let capabilities;
        let (swapchain, images) = {
            capabilities = surface.capabilities(physical.clone()).expect("failed to get surface capabilities");

            dimensions = capabilities.current_extent.unwrap_or([1024, 768]);
            let usage = capabilities.supported_usage_flags;
            let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();
            let format = capabilities.supported_formats[0].0;

            Swapchain::new(device.clone(), surface.clone(), capabilities.min_image_count,
                           format, dimensions, 1, usage, &queue,
                           ::vulkano::swapchain::SurfaceTransform::Identity, alpha,
                           ::vulkano::swapchain::PresentMode::Fifo, true, None).expect("failed to create swapchain")
        };

        let uniform_buffer_pool = CpuBufferPool::<DefaultShaders::vertex::ty::Data>::new(device.clone(), BufferUsage::all());
        let uniform_buffer_pool_lines = CpuBufferPool::<LinesShaders::vertex::ty::Data>::new(device.clone(), BufferUsage::all());

        let depth_buffer = ::vulkano::image::attachment::AttachmentImage::transient(device.clone(), dimensions, D32Sfloat).unwrap();

        let vs = DefaultShaders::vertex::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = DefaultShaders::fragment::Shader::load(device.clone()).expect("failed to create shader module");

        let vs_lines = LinesShaders::vertex::Shader::load(device.clone()).expect("failed to create shader module");
        let fs_lines = LinesShaders::fragment::Shader::load(device.clone()).expect("failed to create shader module");

        let renderpass = Arc::new(
            RenderPassClearedColorWithDepth { color_format: swapchain.format()}
                .build_render_pass(device.clone())
                .unwrap()
        );
        let renderpass_lines = Arc::new(
            RenderPassUnclearedColorWithDepth { color_format: swapchain.format()}
                .build_render_pass(device.clone())
                .unwrap()
        );

        let mut registry = TextureRegistry::new();
        registry.load(queue.clone());

        let sampler = Sampler::new(device.clone(), Filter::Nearest, Filter::Nearest, MipmapMode::Nearest,
            SamplerAddressMode::Repeat, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
            0.0, 4.0, 0.0, 0.0).unwrap();

        let pipeline = Arc::new(GraphicsPipeline::start()
            .cull_mode_back()
            .vertex_input_single_buffer::<VertexPositionNormalUVColor>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .blend_alpha_blending()
            .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap());

        let pipeline_lines = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<VertexPositionColorAlpha>()
            .vertex_shader(vs_lines.main_entry_point(), ())
            .line_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs_lines.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .blend_alpha_blending()
            .render_pass(Subpass::from(renderpass_lines.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap());

        let memory_pool = AutoMemoryPool::new(device.clone());

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
            vertex_buffer: CpuAccessibleBufferAutoPool::<[VertexPositionColorAlpha]>::from_iter(device.clone(), memory_pool.clone(), ::vulkano::buffer::BufferUsage::all(), temp_line_verts.iter().cloned()).expect("failed to create buffer"),
            index_buffer: CpuAccessibleBufferAutoPool::<[u32]>::from_iter(device.clone(), memory_pool.clone(), ::vulkano::buffer::BufferUsage::all(), temp_line_idxs.iter().cloned()).expect("failed to create buffer"),
        };

        Renderer {
            device,
            memory_pool,
            queue,
            surface,
            renderpass,
            renderpass_lines,
            framebuffers: None,
            framebuffers_lines: None,
            swapchain,
            images,
            pipeline,
            pipeline_lines,
            sampler,
            depth_buffer,
            uniform_buffer_pool,
            uniform_buffer_pool_lines,
            recreate_swapchain: false,
            tex_registry: registry,
            temp_line_data
        }
    }


    pub fn draw(&mut self, camera: &Camera, transform: Transform, render_queue: &Vec<RenderQueueMeshEntry>) {
        let view_mat = Matrix4::from(transform.rotation) * Matrix4::from_translation((transform.position * -1.0).to_vec());

        let dimensions = match self.surface.window().get_inner_size() {
            Some(::winit::dpi::LogicalSize{ width, height }) => [width as u32, height as u32],
            None => [1024, 768]
        };

        if self.recreate_swapchain {
            let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => {
                    println!("SwapchainCreationError::UnsupportedDimensions");
                    return;
                },
                Err(err) => panic!("{:?}", err)
            };

            ::std::mem::replace(&mut self.swapchain, new_swapchain);
            ::std::mem::replace(&mut self.images, new_images);

            let new_depth_buffer = AttachmentImage::transient(self.device.clone(), dimensions, D32Sfloat).unwrap();
            ::std::mem::replace(&mut self.depth_buffer, new_depth_buffer);

            self.framebuffers = None;
            self.framebuffers_lines = None;

            self.recreate_swapchain = false;
        }

        if self.framebuffers.is_none() {
            let new_framebuffers = Some(self.images.iter().map(|image| {
                let arc: Arc<FramebufferAbstract + Send + Sync> = Arc::new(Framebuffer::start(self.renderpass.clone())
                    .add(image.clone()).unwrap()
                    .add(self.depth_buffer.clone()).unwrap()
                    .build().unwrap());
                arc
            }).collect::<Vec<_>>());
            ::std::mem::replace(&mut self.framebuffers, new_framebuffers);
        }
        if self.framebuffers_lines.is_none() {
            let new_framebuffers = Some(self.images.iter().map(|image| {
                let arc: Arc<FramebufferAbstract + Send + Sync> = Arc::new(Framebuffer::start(self.renderpass_lines.clone())
                    .add(image.clone()).unwrap()
                    .add(self.depth_buffer.clone()).unwrap()
                    .build().unwrap());
                arc
            }).collect::<Vec<_>>());
            ::std::mem::replace(&mut self.framebuffers_lines, new_framebuffers);
        }

        let mut descriptor_sets = Vec::new();
        for entry in render_queue.iter() {
            let uniform_data = DefaultShaders::vertex::ty::Data {
                world : entry.transform.clone().into(),
                view : view_mat.into(),
                proj : (VULKAN_CORRECT_CLIP * ::cgmath::perspective(camera.fov, { dimensions[0] as f32 / dimensions[1] as f32 }, 0.1, 100.0)).into(),
            };

            let subbuffer = self.uniform_buffer_pool.next(uniform_data).unwrap();
            descriptor_sets.push(Arc::new(::vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                .add_sampled_image(self.tex_registry.get(&entry.material.albedo_map_name).unwrap().clone(), self.sampler.clone()).unwrap()
                .add_buffer(subbuffer).unwrap()
                .build().unwrap()
            ));
        };

        let line_descriptor_set;
        {
            let subbuffer = self.uniform_buffer_pool_lines.next(LinesShaders::vertex::ty::Data {
                world : Matrix4::from_scale(1.0).into(),
                view : view_mat.into(),
                proj : (VULKAN_CORRECT_CLIP * ::cgmath::perspective(camera.fov, { dimensions[0] as f32 / dimensions[1] as f32 }, 0.1, 100.0)).into(),
            }).unwrap();
            line_descriptor_set = Arc::new(::vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(self.pipeline_lines.clone(), 0)
                .add_buffer(subbuffer).unwrap()
                .build().unwrap()
            );
        }

        let (image_num, future) = match ::vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(r) => r,
            Err(::vulkano::swapchain::AcquireError::OutOfDate) => {
                self.recreate_swapchain = true;
                println!("AcquireError::OutOfDate");
                return
            },
            Err(err) => panic!("{:?}", err)
        };

        let mut cb = ::vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family())
            .unwrap()
            .begin_render_pass(
                self.framebuffers.as_ref().unwrap()[image_num].clone(), false,
                vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()]).unwrap();
        for (i, entry) in render_queue.iter().enumerate() {
            cb = cb.draw_indexed(self.pipeline.clone(), ::vulkano::command_buffer::DynamicState {
                line_width: None,
                viewports: Some(vec![::vulkano::pipeline::viewport::Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }]),
                scissors: None,
            },
            vec![entry.vertex_group.vertex_buffer.as_ref().unwrap().clone()],
            entry.vertex_group.index_buffer.as_ref().unwrap().clone(),
            descriptor_sets[i].clone(), ()).unwrap();
        }
        let cb = cb.end_render_pass().unwrap()
                   .build().unwrap();

        let cb_lines = ::vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family())
            .unwrap()
            .begin_render_pass(
                self.framebuffers_lines.as_ref().unwrap()[image_num].clone(), false,
                vec![::vulkano::format::ClearValue::None, ::vulkano::format::ClearValue::None]).unwrap()
            .draw_indexed(self.pipeline_lines.clone(), ::vulkano::command_buffer::DynamicState {
                                 line_width: None,
                                 viewports: Some(vec![::vulkano::pipeline::viewport::Viewport {
                                     origin: [0.0, 0.0],
                                     dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                     depth_range: 0.0..1.0,
                                 }]),
                                 scissors: None,
                             },
                             vec![self.temp_line_data.vertex_buffer.clone()],
                             self.temp_line_data.index_buffer.clone(),
                             line_descriptor_set.clone(), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        let future = future.then_execute(self.queue.clone(), cb).unwrap()
            .then_execute(self.queue.clone(), cb_lines).unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(mut f) => { f.cleanup_finished() }
            Err(::vulkano::sync::FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}
