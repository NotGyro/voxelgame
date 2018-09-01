use std::sync::Arc;

use cgmath::{EuclideanSpace, Matrix4, Vector4};

use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::D32Sfloat;
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::{Swapchain, Surface, SwapchainCreationError};
use vulkano::sync::{GpuFuture};
use winit::Window;

use util::{Camera, Transform};
use geometry::{VertexGroup, Material};
use registry::TextureRegistry;
use pool::AutoMemoryPool;
use pipeline::{ChunkRenderPipeline, LinesRenderPipeline, SkyboxRenderPipeline};


pub static VULKAN_CORRECT_CLIP: Matrix4<f32> = Matrix4 {
    x: Vector4 { x: 1.0, y:  0.0, z: 0.0, w: 0.0 },
    y: Vector4 { x: 0.0, y: -1.0, z: 0.0, w: 0.0 },
    z: Vector4 { x: 0.0, y:  0.0, z: 0.5, w: 0.5 },
    w: Vector4 { x: 0.0, y:  0.0, z: 0.0, w: 1.0 }
};


pub struct ChunkRenderQueueEntry {
    pub vertex_group: Arc<VertexGroup>,
    pub material: Material,
    pub transform: Matrix4<f32>
}


pub struct Renderer {
    pub device: Arc<Device>,
    pub memory_pool: AutoMemoryPool,
    queue: Arc<Queue>,
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    depth_buffer: Arc<AttachmentImage<D32Sfloat>>,
    recreate_swapchain: bool,
    tex_registry: TextureRegistry,
    skybox_pipeline: SkyboxRenderPipeline,
    chunk_pipeline: ChunkRenderPipeline,
    lines_pipeline: LinesRenderPipeline,
    pub chunk_mesh_queue: Vec<ChunkRenderQueueEntry>,
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

            let format;
            if capabilities.supported_formats.contains(&(::vulkano::format::Format::B8G8R8A8Srgb, ::vulkano::swapchain::ColorSpace::SrgbNonLinear)) {
                format = ::vulkano::format::Format::B8G8R8A8Srgb;
            }
            else {
                format = capabilities.supported_formats[0].0;
            }

            Swapchain::new(device.clone(), surface.clone(), capabilities.min_image_count,
                           format, dimensions, 1, usage, &queue,
                           ::vulkano::swapchain::SurfaceTransform::Identity, alpha,
                           ::vulkano::swapchain::PresentMode::Fifo, true, None).expect("failed to create swapchain")
        };

        let depth_buffer = ::vulkano::image::attachment::AttachmentImage::transient(device.clone(), dimensions, D32Sfloat).unwrap();

        let mut registry = TextureRegistry::new();
        registry.load(queue.clone());

        let memory_pool = AutoMemoryPool::new(device.clone());

        let skybox_pipeline = SkyboxRenderPipeline::new(&swapchain, &device, &queue, &memory_pool);
        let chunk_pipeline = ChunkRenderPipeline::new(&swapchain, &device);
        let lines_pipeline = LinesRenderPipeline::new(&swapchain, &device, &memory_pool);

        Renderer {
            device,
            memory_pool,
            queue,
            surface,
            swapchain,
            images,
            depth_buffer,
            recreate_swapchain: false,
            tex_registry: registry,
            skybox_pipeline,
            chunk_pipeline,
            lines_pipeline,
            chunk_mesh_queue: Vec::new()
        }
    }


    pub fn draw(&mut self, camera: &Camera, transform: Transform) {
        let dimensions = match self.surface.window().get_inner_size() {
            Some(::winit::dpi::LogicalSize{ width, height }) => [width as u32, height as u32],
            None => [1024, 768]
        };
        // minimizing window makes dimensions = [0, 0] which breaks swapchain creation.
        // skip draw loop until window is restored.
        if dimensions[0] < 1 || dimensions[1] < 1 { return; }

        let view_mat = Matrix4::from(transform.rotation) * Matrix4::from_translation((transform.position * -1.0).to_vec());
        let proj_mat = VULKAN_CORRECT_CLIP * ::cgmath::perspective(camera.fov, { dimensions[0] as f32 / dimensions[1] as f32 }, 0.1, 100.0);

        if self.recreate_swapchain {
            println!("Recreating swapchain");
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

            self.skybox_pipeline.remove_framebuffers();
            self.chunk_pipeline.remove_framebuffers();
            self.lines_pipeline.remove_framebuffers();

            self.recreate_swapchain = false;
        }

        if self.skybox_pipeline.framebuffers.is_none() {
            self.skybox_pipeline.recreate_framebuffers(&self.images, &self.depth_buffer);
        }
        if self.chunk_pipeline.framebuffers.is_none() {
            self.chunk_pipeline.recreate_framebuffers(&self.images, &self.depth_buffer);
        }
        if self.lines_pipeline.framebuffers.is_none() {
            self.lines_pipeline.recreate_framebuffers(&self.images, &self.depth_buffer);
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

        let skybox_cb = self.skybox_pipeline.build_command_buffer(image_num, &self.queue, dimensions, view_mat, proj_mat);
        let chunks_cb = self.chunk_pipeline.build_command_buffer(image_num, &self.queue, dimensions, &transform, view_mat, proj_mat, &self.tex_registry, &self.chunk_mesh_queue);
        let lines_cb = self.lines_pipeline.build_command_buffer(image_num, &self.queue, dimensions, view_mat, proj_mat);

        let future = future
            .then_execute(self.queue.clone(), skybox_cb).unwrap()
            .then_execute(self.queue.clone(), chunks_cb).unwrap()
            .then_execute(self.queue.clone(), lines_cb).unwrap()
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
