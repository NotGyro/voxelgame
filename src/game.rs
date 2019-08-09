//! Main type for the game. `Game::new().run()` runs the game.
extern crate parking_lot;
extern crate log;

use self::parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Instant, Duration};
use std::collections::HashMap;

use cgmath::Point3;
use vulkano::buffer::BufferUsage;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Event, WindowEvent, DeviceEvent, KeyboardInput, VirtualKeyCode};
use winit::{Window, WindowBuilder};

use buffer::CpuAccessibleBufferAutoPool;
use geometry::VertexPositionColorAlpha;
use geometry::Mesh;
use geometry::Material;
use renderer::Renderer;
use input::InputState;
use world::Dimension;
use registry::DimensionRegistry;
use player::Player;
use world::dimension::{CHUNK_STATE_DIRTY, CHUNK_STATE_WRITING, CHUNK_STATE_CLEAN};

use mesh_simplifier::*;
use voxel::voxelmath::*;
use voxel::voxelstorage::*;

use world::block::Chunk;

/// Naive implementation of something Future-shaped.
type PendingMesh = Arc<Mutex<Option<Mesh>>>;

fn poll_pending_mesh(pend : PendingMesh) -> Option<Mesh> {
    match pend.try_lock() {
        Some(mut guard) => guard.take(),
        None => None,
    }
}

fn complete_pending_mesh(pend : PendingMesh, mesh : Mesh) {
    pend.lock().replace(mesh);
}

fn new_pending_mesh() -> PendingMesh { Arc::new(Mutex::new(None)) }

/// Main type for the game. `Game::new().run()` runs the game.
pub struct Game {
    events_loop: EventsLoop,
    surface: Arc<Surface<Window>>,
    renderer: Renderer,
    prev_time: Instant,
    input_state: InputState,
    player: Player,
    dimension_registry: DimensionRegistry,
    pending_meshes : Vec<(VoxelPos<i32>, PendingMesh, Instant)>,
    chunk_meshes: HashMap<VoxelPos<i32>, Mesh>,
}

impl Game {
    /// Creates a new `Game`.
    pub fn new() -> Game {
        let instance = Instance::new(None, &::vulkano_win::required_extensions(), None).expect("failed to create instance");
        let events_loop = EventsLoop::new();
        let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
        let renderer = Renderer::new(instance.clone(), surface.clone());

        let input_state = InputState::new();

        let mut player = Player::new();
        player.position = Point3::new(16.0, 32.0, 16.0);

        player.yaw = -135.0;
        player.pitch = -30.0;

        let mut dimension_registry = DimensionRegistry::new();
        let dimension = Dimension::new();
        dimension_registry.dimensions.insert(0, dimension);

        let pending_meshes = Vec::new();
        let chunk_meshes = HashMap::new();

        Game {
            events_loop,
            surface,
            renderer,
            prev_time: Instant::now(),
            input_state,
            player,
            dimension_registry,
            pending_meshes,
            chunk_meshes,
        }
    }


    /// Runs the main game loop.
    pub fn run(&mut self) {
        let mut running = true;
        while running {
            running = self.update();
        }
    }


    /// Main game loop.
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
                Event::DeviceEvent { event: DeviceEvent::Key(inp), .. }  => {
                    self.input_state.update_key(inp);
                    if(inp.virtual_keycode == Some(VirtualKeyCode::Escape)) {
                        keep_running = false;
                    }
                    if(inp.virtual_keycode == Some(VirtualKeyCode::E)) {
                        println!("{:?}", self.player.position);
                        let pos = vpos!(self.player.position.x as i32, self.player.position.y as i32, self.player.position.z as i32);
                        self.dimension_registry.get(0).unwrap().set(pos, 0);
                    }

                },
                _ => ()
            }
        }

        self.player.update(dt, &self.input_state);

        self.dimension_registry.get(0).unwrap().load_unload_chunks(self.player.position.clone(), &mut self.renderer.render_queue.lines);
        
        {
            let line_queue = &mut self.renderer.render_queue.lines;
            if line_queue.chunks_changed {
                let mut verts = Vec::new();
                let mut idxs = Vec::new();
                let mut index_offset = 0;
                for (pos, _) in self.dimension_registry.get(0).unwrap().chunks.iter() {
                    verts.append(&mut ::util::cube::generate_chunk_debug_line_vertices(pos.x, pos.y, pos.z, 0.25f32).to_vec());
                    idxs.append(&mut ::util::cube::generate_chunk_debug_line_indices(index_offset).to_vec());
                    index_offset += 1;
                }
                line_queue.chunk_lines_vertex_buffer =
                    CpuAccessibleBufferAutoPool::<[VertexPositionColorAlpha]>::from_iter(self.renderer.device.clone(),
                                                                                         self.renderer.memory_pool.clone(),
                                                                                         BufferUsage::all(),
                                                                                         verts.iter().cloned())
                        .expect("failed to create buffer");
                line_queue.chunk_lines_index_buffer =
                    CpuAccessibleBufferAutoPool::<[u32]>::from_iter(self.renderer.device.clone(),
                                                                    self.renderer.memory_pool.clone(),
                                                                    BufferUsage::all(),
                                                                    idxs.iter().cloned())
                        .expect("failed to create buffer");
                line_queue.chunks_changed = false;
            }
        }

        let loaded_chunk_list = self.dimension_registry.get(0).unwrap().loaded_chunk_list();

        let chunk_size = self.dimension_registry.get(0).unwrap().chunk_size;
        self.renderer.render_queue.chunk_meshes.clear();
        for (pos, (ref mut entry)) in self.dimension_registry.get(0).unwrap().chunks.iter_mut() {
            let is_dirty = entry.state.load(Ordering::Relaxed) == CHUNK_STATE_DIRTY;
            if is_dirty {
                entry.state.store(CHUNK_STATE_WRITING, Ordering::Relaxed);
                let entry_arc = entry.clone();

                let device_arc = self.renderer.device.clone();
                let memory_pool_arc = self.renderer.memory_pool.clone();

                let mut mesh_pend = new_pending_mesh();
                self.pending_meshes.push((*pos, mesh_pend.clone(), Instant::now()));

                let chunk_origin = entry_arc.bounds.lower.clone();
                let bounds = entry_arc.bounds.clone();
                
                thread::spawn(move || {
                    let chunk_lock = entry_arc.data.read();
                    let mut mesh = MeshSimplifier::generate_mesh(&*chunk_lock as &Chunk, bounds, device_arc, memory_pool_arc).unwrap();

                    mesh.materials.push(Material { albedo_map_name: String::from(""), specular_exponent: 0.0, specular_strength: 0.6 });
                    mesh.materials.push(Material { albedo_map_name: String::from("stone"), specular_exponent: 128.0, specular_strength: 1.0 });
                    mesh.materials.push(Material { albedo_map_name: String::from("dirt"), specular_exponent: 16.0, specular_strength: 0.5 });
                    mesh.materials.push(Material { albedo_map_name: String::from("grass"), specular_exponent: 64.0, specular_strength: 0.7 });

                    complete_pending_mesh(mesh_pend.clone(), mesh);
                    entry_arc.state.store(CHUNK_STATE_CLEAN, Ordering::Relaxed);
                });
            }
        }
        let mut new_meshes: Vec<(VoxelPos<i32>, Mesh)> = Vec::new();
        // Add any mesh from a task that just finished.
        self.pending_meshes.retain(|(pos, pending_mesh, time)| {
            match poll_pending_mesh(pending_mesh.clone()) {
                Some(mesh) => { //Mesh is done! Remove it from this list.
                    new_meshes.push((*pos, mesh));
                    trace!("Chunk mesh at ({}, {}, {}) took {} milliseconds to generate.", pos.x, pos.y, pos.z, time.elapsed().as_millis());
                    false
                }
                None => true, //Not done yet, keep this around to poll again next time.
            }
        });
        for elem in new_meshes.drain(..) {
            self.chunk_meshes.insert(elem.0, elem.1);
        }

        // Clean up meshes for chunks that are no longer loaded.
        self.chunk_meshes.retain(|pos, _ | { loaded_chunk_list.contains(pos) } );

        // Actually add the mesh to our render queue.
        for mesh in self.chunk_meshes.values_mut() {
            self.renderer.render_queue.chunk_meshes.append(&mut mesh.queue());
        }

        self.renderer.draw(&self.player.camera, self.player.get_transform());

        //println!("{:?}", self.player.get_transform());
        return keep_running;
    }
}
