use std::sync::Arc;
use std::time::Instant;

use cgmath::Point3;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Event, WindowEvent, DeviceEvent};
use winit::{Window, WindowBuilder};

use renderer::Renderer;
use input::InputState;
use world::Dimension;
use registry::DimensionRegistry;
use player::Player;


pub struct Game {
    events_loop: EventsLoop,
    surface: Arc<Surface<Window>>,
    renderer: Renderer,
    prev_time: Instant,
    input_state: InputState,
    player: Player,
    dimension_registry: DimensionRegistry
}


impl Game {
    pub fn new() -> Game {
        let instance = Instance::new(None, &::vulkano_win::required_extensions(), &[]).expect("failed to create instance");
        let events_loop = EventsLoop::new();
        let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
        let renderer = Renderer::new(instance.clone(), surface.clone());

        let input_state = InputState::new();

        let mut player = Player::new();
        player.position = Point3::new(16.0, 16.0, 16.0);
        player.yaw = 135.0;
        player.pitch = -30.0;

        let mut dimension_registry = DimensionRegistry::new();
        let dimension = Dimension::new();
        dimension_registry.dimensions.insert(0, dimension);

        Game {
            events_loop,
            surface,
            renderer,
            prev_time: Instant::now(),
            input_state,
            player,
            dimension_registry
        }
    }


    pub fn run(&mut self) {
        let mut running = true;
        while running {
            running = self.update();
        }
    }


    pub fn update(&mut self) -> bool {
        let mut keep_running = true;

        let elapsed = Instant::now() - self.prev_time;
        let dt = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;
        self.prev_time = Instant::now();

        self.input_state.mouse_delta = (0.0, 0.0);

        let mut events = Vec::new() as Vec<Event>;
        self.events_loop.poll_events(|ev| { events.push(ev); });

        for ev in events {
            match ev {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => keep_running = false,
                        WindowEvent::KeyboardInput {input, ..} => self.input_state.update_key(input),
                        _ => {}
                    }
                },
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                    self.input_state.add_mouse_delta(delta);
                    if self.input_state.right_mouse_pressed {
                        let dimensions = match self.surface.window().get_inner_size() {
                            Some(::winit::dpi::LogicalSize{ width, height }) => [width as u32, height as u32],
                            None => [1024, 768]
                        };
                        match self.surface.window().set_cursor_position(::winit::dpi::LogicalPosition::new(dimensions[0] as f64 / 2.0, dimensions[1] as f64 / 2.0)) {
                            Ok(_) => {},
                            Err(err) => { println!("Couldn't set cursor position: {:?}", err); }
                        }
                    }
                },
                Event::DeviceEvent { event: DeviceEvent::Button { button, state }, .. } => {
                    if button == 3 {
                        match state {
                            ::winit::ElementState::Pressed => {
                                self.surface.window().hide_cursor(true);
                                self.input_state.right_mouse_pressed = true;
                            },
                            ::winit::ElementState::Released => {
                                self.surface.window().hide_cursor(false);
                                self.input_state.right_mouse_pressed = false;
                            }
                        }
                    }
                },
                _ => ()
            }
        }

        self.player.update(dt, &self.input_state);

        self.dimension_registry.get(0).unwrap().load_unload_chunks(self.player.position.clone());

        self.renderer.chunk_mesh_queue.clear();
        for (_, mut chunk) in self.dimension_registry.get(0).unwrap().chunks.iter_mut() {
            if chunk.mesh_dirty {
                chunk.generate_mesh(&self.renderer);
            }
        }
        for (_, chunk) in self.dimension_registry.get(0).unwrap().chunks.iter() {
            self.renderer.chunk_mesh_queue.append(&mut chunk.mesh.queue());
        }
        self.renderer.draw(&self.player.camera, self.player.get_transform());

        return keep_running;
    }
}