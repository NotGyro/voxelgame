//! Main type for the game. `Game::new().run()` runs the game.
extern crate parking_lot;
extern crate log;
extern crate crossbeam;
extern crate serde;
extern crate serde_json;

use self::parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::result::Result;
use std::error;
use std::ops::Neg;

//use std::net::{IpAddr, SocketAddr, TcpStream, TcpListener};
use std::net::SocketAddr;

use cgmath::{Point3, Rotation, Rotation3, Quaternion, Deg, Rad, Vector3, InnerSpace};
use vulkano::buffer::BufferUsage;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Event, WindowEvent, DeviceEvent, VirtualKeyCode};
use winit::{Window, WindowBuilder};

use buffer::CpuAccessibleBufferAutoPool;
use geometry::VertexPositionColorAlpha;
use geometry::Mesh;
use geometry::Material;
use renderer::Renderer;
use input::InputState;
use world::Dimension;
use registry::DimensionRegistry;
use player::PlayerController;
use world::dimension::{CHUNK_STATE_DIRTY, CHUNK_STATE_WRITING, CHUNK_STATE_CLEAN};

use mesh_simplifier::*;
use voxel::voxelmath::*;
use voxel::voxelstorage::*;
use voxel::voxelevent::*;

use util::logger::*;
use util::event::*;

use world::block::Chunk;
use world::block::BlockID;

use network;

//use self::crossbeam::crossbeam_channel::{unbounded, after};
use self::crossbeam::crossbeam_channel::{Sender, Receiver};
//use self::bincode::deserialize_from;
//use self::bincode::serialize_into;

//use serde::{Serialize, Deserialize};

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

pub type PlayerID = u64;
pub type Port = u16;

pub type PlayerPosition = (f32, f32, f32);

#[derive(PartialEq, Eq)]
pub enum GameMode {
    Singleplayer,
    JoinServer(SocketAddr),
    Server(SocketAddr),
}

pub struct GameClient {
    events_loop: EventsLoop,
    surface: Arc<Surface<Window>>,
    renderer: Renderer,
    prev_time: Instant,
    input_state: InputState,
    player: PlayerController,
    pending_meshes : Vec<(VoxelPos<i32>, PendingMesh, Instant)>,
    chunk_meshes: HashMap<VoxelPos<i32>, Mesh>,
    voxel_event_sender : Sender<VoxelEvent<BlockID, i32>>,
    voxel_event_receiver : Receiver<VoxelEvent<BlockID, i32>>,
    net: network::Client,
}

/// Main type for the game. `Game::new().run()` runs the game.
pub struct Game {
    dimension_registry: DimensionRegistry,
    event_bus: SimpleEventBus<VoxelEvent<BlockID, i32>>,
    voxel_event_sender : Sender<VoxelEvent<BlockID, i32>>,
    voxel_event_receiver : Receiver<VoxelEvent<BlockID, i32>>,
    current_server_tick : u64,
    last_tick: Instant,
    since_tick: Duration,
    c: Option<GameClient>,
    net_srv: Option<network::Server>,
    mode: GameMode,
}

impl Game {
    /// Creates a new `Game`.
    pub fn new(mode : GameMode) -> Game {

        let is_server = match mode {
            GameMode::Server(_) => true,
            _ => false,
        };
        let since_tick = Duration::new(0,0);
        let last_tick = Instant::now();

        let mut dimension_registry = DimensionRegistry::new();
        let dimension = Dimension::new();
        dimension_registry.dimensions.insert(0, dimension);
        let mut bus : SimpleEventBus<VoxelEvent<BlockID, i32>> = SimpleEventBus::new();
        
        let sender = bus.get_sender();
        let (receiver, _) = bus.subscribe(); // We don't need the ID since we're never going to remove this channel until the game terminates. 

        if !is_server {
            // We are singleplayer or joining a server, 
            let instance = Instance::new(None, &::vulkano_win::required_extensions(), None).expect("failed to create instance");
            let events_loop = EventsLoop::new();
            let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
            let renderer = Renderer::new(instance.clone(), surface.clone());

            let input_state = InputState::new();

            let mut player = PlayerController::new();
            player.position = Point3::new(16.0, 32.0, 16.0);

            player.yaw = -135.0;
            player.pitch = -30.0;

            let pending_meshes = Vec::new();
            let chunk_meshes = HashMap::new();

            let voxel_event_sender = sender.clone();
            let (voxel_event_receiver, _) = bus.subscribe(); // We don't need the ID since we're never going to remove this channel until the game terminates.
            surface.window().hide_cursor(true);
            let mut net = network::Client::new();
            if let GameMode::JoinServer(addr) = mode {
                net.connect(addr).unwrap();
            }

            return Game {
                dimension_registry: dimension_registry,
                event_bus : bus,
                voxel_event_sender : sender,
                voxel_event_receiver : receiver,
                current_server_tick : 0,
                last_tick : last_tick, 
                since_tick : since_tick,
                c : Some(GameClient {
                    events_loop,
                    surface,
                    renderer,
                    prev_time: Instant::now(),
                    input_state,
                    player,
                    pending_meshes,
                    chunk_meshes,
                    voxel_event_sender,
                    voxel_event_receiver,
                    net,
                }),
                net_srv : None,
                mode : mode,
            };
        }
        else { 
            if let GameMode::Server(addr) = mode {
                //thread::spawn( move || { start_server(addr).map_err(|err| {error!("{}", err)}) } );
                return Game {
                    dimension_registry: dimension_registry,
                    event_bus : bus,
                    voxel_event_sender : sender,
                    voxel_event_receiver : receiver,
                    current_server_tick : 0,
                    last_tick : last_tick, 
                    since_tick : since_tick,
                    c : None,
                    net_srv : Some(network::Server::new(addr).map_err( |err|
                                 {error!("{}", err); panic!();}).unwrap()),
                    mode : mode,
                };
            }
            unreachable!();
        }
    }

    /// Runs the main game loop.
    pub fn run(&mut self) {
        const TICK_LENGTH : Duration = Duration::from_millis(50); //Length of a single tick in milliseconds
        let mut running = true;

        while running {
            //Primary glue code for networking goes here. 
            //This is so that singleplayer vs joining a server is transparent to the client,
            //and having a client is (mostly) transparent to the server.

            //Serverside chunk stuff.
            if let GameMode::Server(_ip) = self.mode {
                //let player_positions = self.players.iter().map(|player| { player.pos.into() }).collect();
                self.dimension_registry.get_mut(0).unwrap().load_unload_chunks_clientside(Point3{x:0.0,y:0.0,z:0.0});
            }

            //Handle networking if we're a server.
            if let Some(ref mut srv) = self.net_srv {
                match srv.accept_step() {
                    Ok(_) => {}, 
                    Err(err) => {error!("Error in accept step of network system: {}", err); panic!();},
                }
                match srv.stream_step() {
                    Ok(_) => {}, 
                    Err(err) => {error!("Error in stream step of network system: {}", err); panic!();},
                }
                match srv.cleanup_step() {
                    Ok(_) => {}, 
                    Err(err) => {error!("Error in cleanup step of network system: {}", err); panic!();},
                }
            }
            //Process server ticks
            let elapsed = Instant::now() - self.last_tick;
            self.last_tick = Instant::now();
            self.since_tick += elapsed;

            let mut events_from_clients : Vec<(network::Identity, VoxelEvent<BlockID, i32>)> = Vec::new();
            // Handle voxel events we got from these clients.
            if self.net_srv.is_some() {
                let mut srv = self.net_srv.take().unwrap();
                for pak in srv.poll() {
                    if let network::ToServerPacketData::VoxEv(event) = pak.pak.data {
                        //Route voxel events through our own instance of the engine.
                        self.voxel_event_sender.send(event.clone()).unwrap();
                        // Queue this event to see if it's valid.
                        events_from_clients.push((pak.client_id, event.clone()));
                    }
                }
                //Put it back.
                self.net_srv = Some(srv);
            }

            while self.since_tick >= TICK_LENGTH {
                // Let the logger know what tick it is.
                let mut gls = GAME_LOGGER_STATE.lock();
                gls.current_tick = self.current_server_tick;
                drop(gls);
                // Increment our current server tick and decrement how much "to-tick" time we've got.
                self.current_server_tick += 1;
                self.since_tick -= TICK_LENGTH;
            }
            // Move our Voxel Events along.
            self.event_bus.process();
            for event in self.voxel_event_receiver.try_iter().collect::<Vec<VoxelEvent<BlockID, i32>>>(){
                trace!("Got event: {:?}", event); 
                match self.dimension_registry.get_mut(0).unwrap().apply_event(event.clone()) {
                    Ok(_) => {
                        // We have succeeded in applying this event to our world, so it's valid. Record it, tell the players about it.
                        //self.event_history.push(event.clone());
                        //Send to clients if we're a server.
                        if self.net_srv.is_some() {
                            let mut srv = self.net_srv.take().unwrap();
                            for pak in srv.poll() {
                                if let network::ToServerPacketData::VoxEv(event) = pak.pak.data {
                                    srv.queue_broadcast(
                                        network::QualifiedToClientPacket{client_id:pak.client_id, 
                                            pak: network::ToClientPacket {
                                                data: network::ToClientPacketData::VoxEv(event),
                                    },});
                                }
                            }
                            //Put it back.
                            self.net_srv = Some(srv);
                        }
                    },
                    Err(error) => { 
                        match error {
                            VoxelError::NotYetLoaded(pos) => warn!("Attempted to access an unloaded voxel at {}", pos),
                            _ => {error!("Received an error when attempting to apply a voxel event: {}", error); return;},
                        }
                    },
                }
            }

            // Do clientsided things.
            if self.c.is_some() {
                let mut client = self.c.take().unwrap();
                #[allow(unused_mut)] //This will probably need to be mutable in the future.
                self.dimension_registry.get_mut(0).unwrap().load_unload_chunks_clientside(client.player.position.clone());
                match client.update(&self.dimension_registry) {
                    Ok(keep_running) => running = keep_running,
                    Err(error) => error!("Encountered an error in tick {} in client mainloop: {}", self.current_server_tick, error),
                }  
                self.c = Some(client); // Take ownership again
            }
        }
    }
}

impl GameClient {
    /// Main game loop.
    pub fn update(&mut self, dimension_registry : &DimensionRegistry) -> Result<bool, Box<dyn error::Error>> {
        let mut keep_running = true;

        let elapsed = Instant::now() - self.prev_time;
        let dt = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;
        self.prev_time = Instant::now();

        self.input_state.mouse_delta = (0.0, 0.0);

        let mut events = Vec::new() as Vec<Event>;
        self.events_loop.poll_events(|ev| { events.push(ev); });

        let yaw = Deg::<f32>(self.player.yaw as f32);
        let pitch = Deg::<f32>(self.player.pitch.neg() as f32);

        let yawq : Quaternion<f32> = Quaternion::from_angle_y(Rad::<f32>::from(yaw));
        let pitchq : Quaternion<f32> = Quaternion::from_angle_x(Rad::<f32>::from(pitch));
        let rotation = (yawq * pitchq).normalize();

        let mut forward : Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);
        forward = rotation.rotate_vector(forward);
        forward.z = forward.z.neg();

        let winpos = self.surface.window().get_inner_size().unwrap();
        self.surface.window().set_cursor_position(winit::dpi::LogicalPosition::new(winpos.width * 0.5, winpos.height * 0.5))?;
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
                    // 1 is left mouse, 2 is middle mouse, 3 is right mouse.
                    match button {
                        1 => match state {
                            ::winit::ElementState::Pressed => {
                                //self.surface.window().hide_cursor(true);
                                self.input_state.left_mouse_pressed = true;
                            },
                            ::winit::ElementState::Released => {
                                //self.surface.window().hide_cursor(false);
                                /*let pos = vpos!(self.player.position.x.floor() as i32, 
                                                self.player.position.y.floor() as i32, 
                                                self.player.position.z.floor() as i32);*/
                                self.input_state.left_mouse_pressed = false;
                                let mut raycast = VoxelRaycast::new(self.player.position, forward);
                                let mut continue_raycast = true;
                                while continue_raycast {
                                    match dimension_registry.get(0).unwrap().get(raycast.pos) {
                                        Ok(voxel) => {
                                            // Is it not air?
                                            if voxel != 0 {
                                                let event = VoxelEvent::SetOne(OneVoxelChange{ new_value : 0, pos : raycast.pos});
                                                self.voxel_event_sender.try_send(event.clone())?;
                                                continue_raycast = false;

                                                //Let the server know (if we're connected to one).
                                                self.net.send_packet(network::ToServerPacket{
                                                    data: network::ToServerPacketData::VoxEv(event.clone())})?;
                                            }
                                        },
                                        Err(_) => continue_raycast = false, //We've left the currently-loaded chunks.
                                    }
                                    raycast.step();
                                }
                            }
                        },
                        2 => match state {
                            ::winit::ElementState::Pressed => {},
                            ::winit::ElementState::Released => {
                                let mut raycast = VoxelRaycast::new(self.player.position, forward);
                                let mut continue_raycast = true;
                                //let mut counter = 0;
                                while continue_raycast {
                                    match dimension_registry.get(0).unwrap().get(raycast.pos) {
                                        Ok(voxel) => {
                                            // Is it not air?
                                            if voxel != 0 {
                                                self.player.selected_block = voxel;
                                                continue_raycast = false;
                                            }
                                        },
                                        Err(_) => continue_raycast = false, //We've left the currently-loaded chunks.
                                    }
                                    raycast.step();
                                }
                            },
                        }
                        3 => match state {
                            ::winit::ElementState::Pressed => {
                                //self.surface.window().hide_cursor(true);
                                self.input_state.right_mouse_pressed = true;
                            },
                            ::winit::ElementState::Released => {
                                //self.surface.window().hide_cursor(false);
                                self.input_state.right_mouse_pressed = false;
                                /*let one_in_front = self.player.position + forward;
                                let block_forward = vpos!(one_in_front.x as i32, one_in_front.y as i32, one_in_front.z as i32);
                                self.voxel_event_sender.try_send(VoxelEvent::SetOne(OneVoxelChange{ new_value : 1, pos : block_forward}))?;*/
                                let mut raycast = VoxelRaycast::new(self.player.position, forward);
                                let mut continue_raycast = true;
                                //let mut counter = 0;
                                while continue_raycast {
                                    match dimension_registry.get(0).unwrap().get(raycast.pos) {
                                        Ok(voxel) => {
                                            // Is it not air?
                                            if voxel != 0 {
                                                let adjacent_pos = raycast.pos.get_neighbor(raycast.get_last_direction().opposite());
                                                let event = VoxelEvent::SetOne(OneVoxelChange{ new_value : self.player.selected_block, pos : adjacent_pos});
                                                self.voxel_event_sender.try_send(event.clone())?;
                                                continue_raycast = false;
                                                //Let the server know (if we're connected to one).
                                                self.net.send_packet(network::ToServerPacket{
                                                    data: network::ToServerPacketData::VoxEv(event.clone())})?;
                                            }
                                        },
                                        Err(_) => continue_raycast = false, //We've left the currently-loaded chunks.
                                    }
                                    raycast.step();
                                }
                            }
                        },
                        _ => {},
                    }
                },
                Event::DeviceEvent { event: DeviceEvent::Key(inp), .. }  => {
                    self.input_state.update_key(inp);
                    if inp.virtual_keycode == Some(VirtualKeyCode::Escape) {
                        keep_running = false;
                    } 
                    if inp.virtual_keycode == Some(VirtualKeyCode::E) && inp.state == ::winit::ElementState::Pressed {
                        println!("{:?}", self.player.position);
                    }
                },
                _ => ()
            }
        }

        self.player.update(dt, &self.input_state);

        {
            let line_queue = &mut self.renderer.render_queue.lines;
            if line_queue.chunks_changed {
                let mut verts = Vec::new();
                let mut idxs = Vec::new();
                let mut index_offset = 0;
                for (pos, _) in dimension_registry.get(0).unwrap().chunks.iter() {
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

        let loaded_chunk_list = dimension_registry.get(0).unwrap().loaded_chunk_list();

        self.renderer.render_queue.chunk_meshes.clear();
        for (pos, ref mut entry) in dimension_registry.get(0).unwrap().chunks.iter() {
            let is_dirty = entry.state.load(Ordering::Relaxed) == CHUNK_STATE_DIRTY;
            if is_dirty {
                entry.state.store(CHUNK_STATE_WRITING, Ordering::Relaxed);
                let entry_arc = entry.clone();

                let device_arc = self.renderer.device.clone();
                let memory_pool_arc = self.renderer.memory_pool.clone();

                let mesh_pend = new_pending_mesh();
                self.pending_meshes.push((*pos, mesh_pend.clone(), Instant::now()));

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
        self.pending_meshes.retain(|(pos, pending_mesh, _time)| {
            match poll_pending_mesh(pending_mesh.clone()) {
                Some(mesh) => { //Mesh is done! Remove it from this list.
                    new_meshes.push((*pos, mesh));
                    //trace!("Chunk mesh at ({}, {}, {}) took {} milliseconds to generate.", pos.x, pos.y, pos.z, time.elapsed().as_millis());
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
        return Ok(keep_running);
    }
}