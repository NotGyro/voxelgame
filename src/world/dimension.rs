//! A dimension.

extern crate parking_lot;
use self::parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use std::collections::HashMap;
use cgmath::{Point3, MetricSpace};
use world::generators::{WorldGenerator, PerlinGenerator};
use voxel::voxelstorage::*;
use voxel::voxelarray::*;
use voxel::voxelmath::*;
use world::block::{BlockID, Chunk};

// TODO: Rewrite this to use the standard Futures API.
/// State used for multithreaded chunk loading. Chunk is dirty and needs to be generated.
pub static CHUNK_STATE_DIRTY: usize = 0;
/// State used for multithreaded chunk loading. Chunk mesh is currently being generated.
pub static CHUNK_STATE_WRITING: usize = 1;
/// State used for multithreaded chunk loading. Chunk is finished being generated.
pub static CHUNK_STATE_CLEAN: usize = 2;

pub struct ChunkEntry { 
    pub data: RwLock<Chunk>,
    pub state: AtomicUsize,
    pub bounds: VoxelRange<i32>,
}

/// A dimension.
pub struct Dimension {
    pub chunks: HashMap<VoxelPos<i32>, Arc<ChunkEntry>>,
    pub chunk_size: VoxelSize<u32>,
}

pub fn blockpos_to_chunk(point: VoxelPos<i32>, chunk_size : VoxelSize<u32>) -> VoxelPos<i32> { 
    vpos!(point.x / chunk_size.x as i32, 
        point.y / chunk_size.y as i32, 
        point.z / chunk_size.z as i32)
}

pub fn chunkpos_to_block(point: VoxelPos<i32>, chunk_size : VoxelSize<u32>) -> VoxelPos<i32> { 
    vpos!(point.x * chunk_size.x as i32, 
        point.y * chunk_size.y as i32, 
        point.z * chunk_size.z as i32)
}

pub fn chunkpos_to_center(point: VoxelPos<i32>, chunk_size : VoxelSize<u32>) -> Point3<f32> { 
    let block_pos = chunkpos_to_block(point, chunk_size);
    Point3::new(block_pos.x as f32 + (chunk_size.x as f32 * 0.5), 
        block_pos.y as f32 + (chunk_size.y as f32 * 0.5), 
        block_pos.z as f32 + (chunk_size.z as f32 * 0.5))
}

impl VoxelStorage<BlockID, i32> for Dimension {
    fn get(&self, coord: VoxelPos<i32>) -> Option<BlockID>{
        let size = self.chunk_size.clone();
        // Do we have a chunk that would contain this block position?
        match self.chunks.get(&blockpos_to_chunk(coord, size)) {
            Some(chunk_entry_arc) => {
                let chunk_entry = chunk_entry_arc.clone();
                let bounds = chunk_entry.bounds.clone();
                assert!(bounds.get_size_unsigned() == size);
                match bounds.get_local_unsigned(coord) {
                    Some(pos) => {
                        // Block until we can get a valid voxel.
                        let locked = chunk_entry.data.read();
                        return locked.get(vpos!(pos.x as u8, pos.y as u8, pos.z as u8));
                    },
                    // Position is not inside our chunk's bounds.
                    None => return None,
                }
            },
            // Chunk not currently loaded or generated.
            None => return None,
        }
    }
    fn set(&mut self, coord: VoxelPos<i32>, value: BlockID) {
        let size = self.chunk_size.clone();
        // Do we have a chunk that would contain this block position?
        let rslt = self.chunks.get(&blockpos_to_chunk(coord, size)).cloned();
        match rslt {
            Some(chunk_entry) => {
                let bounds = chunk_entry.bounds.clone();
                assert!(bounds.get_size_unsigned() == size);
                match bounds.get_local_unsigned(coord) {
                    Some(pos) => {
                        // Block until we can write.
                        let mut locked = chunk_entry.data.write();
                        let position = vpos!(pos.x as u8, pos.y as u8, pos.z as u8);
                        let current = locked.get(position);
                        if current.is_some() && (current.unwrap() != value) {
                            chunk_entry.state.store(CHUNK_STATE_DIRTY, Ordering::Relaxed); //Mark for remesh.
                            locked.set(position, value);
                        }
                    },
                    // Position is not inside our chunk's bounds.
                    None => return,
                }
            },
            // Chunk not currently loaded or generated.
            None => return,
        }
    }
}

impl Dimension {
    pub fn new() -> Dimension {
        Dimension {
            chunks: HashMap::new(),
            chunk_size: vpos!(16, 16, 16),
        }
    }

    pub fn is_chunk_loaded(&self, chunk_pos : VoxelPos<i32> ) -> bool {self.chunks.contains_key(&chunk_pos)}

    pub fn loaded_chunk_list(&self) -> Vec<VoxelPos<i32>> {
        let mut result = Vec::new();
        for pos in self.chunks.keys() {
            result.push(*pos);
        }
        result
    }

    /// Adds new chunks as the player moves closer to them, and removes old chunks as the player
    /// moves away.
    pub fn load_unload_chunks_clientside(&mut self, player_pos: Point3<f32>) {
        const CHUNK_RADIUS: i32 = 2;
        const CHUNK_DISTANCE: f32 = CHUNK_RADIUS as f32 * 2.0 * 16.0;
        const RETAIN_RADIUS: f32 = CHUNK_DISTANCE + 4.0; // offset added to prevent load/unload loop on the edge

        let gen = PerlinGenerator::new();

        let chunk_size = self.chunk_size.clone();
        
        self.chunks.retain(|pos, _| {
            let chunk_pos = chunkpos_to_center(*pos, chunk_size);
            let dist = Point3::distance(chunk_pos, player_pos);
            dist < RETAIN_RADIUS // offset added to prevent load/unload loop on the edge
        });

        let player_x_in_chunks = (player_pos.x / (self.chunk_size.x as f32)) as i32;
        let player_y_in_chunks = (player_pos.y / (self.chunk_size.y as f32)) as i32;
        let player_z_in_chunks = (player_pos.z / (self.chunk_size.z as f32)) as i32;
        for cx in (player_x_in_chunks-CHUNK_RADIUS)..(player_x_in_chunks+CHUNK_RADIUS+1) {
            for cy in (player_y_in_chunks-CHUNK_RADIUS)..(player_y_in_chunks+CHUNK_RADIUS+1) {
                for cz in (player_z_in_chunks-CHUNK_RADIUS)..(player_z_in_chunks+CHUNK_RADIUS+1) {
                    let chunk_pos = vpos!(cx, cy, cz);
                    if self.chunks.contains_key(&chunk_pos) {
                        continue;
                    }

                    let chunk_world_pos = chunkpos_to_center(vpos!(cx, cy, cz), chunk_size);
                    let dist = Point3::distance(chunk_world_pos, player_pos);
                    if dist < CHUNK_DISTANCE {
                        let chunk_origin = chunkpos_to_block(vpos!(cx, cy, cz), chunk_size);
                        let mut range = VoxelRange{lower: chunk_origin, 
                                upper : chunk_origin + vpos!(self.chunk_size.x as i32, self.chunk_size.y as i32, self.chunk_size.z as i32)};
                        range.validate();
                        let mut chunk = gen.generate(range.clone(), 0);
                        self.chunks.insert(chunk_pos, Arc::new(
                            ChunkEntry { 
                                data: RwLock::new(chunk),
                                state: AtomicUsize::new(CHUNK_STATE_DIRTY),
                                bounds: range.clone(),
                            }
                        ));
                        //queue.chunks_changed = true;
                    }
                }
            }
        }
    }
    pub fn load_unload_chunks_serverside(&mut self, player_positions: Vec<Point3<f32>>) {
        const CHUNK_RADIUS: i32 = 2;
        const CHUNK_DISTANCE: f32 = CHUNK_RADIUS as f32 * 2.0 * 16.0;
        const RETAIN_RADIUS: f32 = CHUNK_DISTANCE + 4.0; // offset added to prevent load/unload loop on the edge

        let gen = PerlinGenerator::new();

        let chunk_size = self.chunk_size.clone();
        
        self.chunks.retain(|pos, _| {
            let mut keep : bool = false;
            for player_pos in player_positions.iter() {
                let chunk_pos = chunkpos_to_center(*pos, chunk_size);
                let dist = Point3::distance(chunk_pos, *player_pos);
                if dist < RETAIN_RADIUS {
                    keep = true;
                }
            }
            keep
        });

        for player_pos_ref in player_positions.iter() {
            let player_pos = *player_pos_ref;
            let player_x_in_chunks = (player_pos.x / (self.chunk_size.x as f32)) as i32;
            let player_y_in_chunks = (player_pos.y / (self.chunk_size.y as f32)) as i32;
            let player_z_in_chunks = (player_pos.z / (self.chunk_size.z as f32)) as i32;
            for cx in (player_x_in_chunks-CHUNK_RADIUS)..(player_x_in_chunks+CHUNK_RADIUS+1) {
                for cy in (player_y_in_chunks-CHUNK_RADIUS)..(player_y_in_chunks+CHUNK_RADIUS+1) {
                    for cz in (player_z_in_chunks-CHUNK_RADIUS)..(player_z_in_chunks+CHUNK_RADIUS+1) {
                        let chunk_pos = vpos!(cx, cy, cz);
                        if self.chunks.contains_key(&chunk_pos) {
                            continue;
                        }

                        let chunk_world_pos = chunkpos_to_center(vpos!(cx, cy, cz), chunk_size);
                        let dist = Point3::distance(chunk_world_pos, player_pos);
                        if dist < CHUNK_DISTANCE {
                            let chunk_origin = chunkpos_to_block(vpos!(cx, cy, cz), chunk_size);
                            let mut range = VoxelRange{lower: chunk_origin, 
                                    upper : chunk_origin + vpos!(self.chunk_size.x as i32, self.chunk_size.y as i32, self.chunk_size.z as i32)};
                            range.validate();
                            let mut chunk = gen.generate(range.clone(), 0);
                            self.chunks.insert(chunk_pos, Arc::new(
                                ChunkEntry { 
                                    data: RwLock::new(chunk),
                                    state: AtomicUsize::new(CHUNK_STATE_DIRTY),
                                    bounds: range.clone(),
                                }
                            ));
                            //queue.chunks_changed = true;
                        }
                    }
                }
            }
        }
    }
}