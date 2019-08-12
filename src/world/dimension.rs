//! A dimension.

extern crate parking_lot;

use self::parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::error::Error;
use std::fmt;

use std::collections::HashMap;
use cgmath::{Point3, MetricSpace};
use world::generators::{WorldGenerator, PerlinGenerator};
use voxel::voxelstorage::*;
use voxel::voxelarray::*;
use voxel::voxelmath::*;
use world::block::{BlockID, Chunk};

/// An error reported upon trying to get or set a voxel which is not currently loaded. 
#[derive(Debug, Copy, Clone)]
pub enum ChunkedVoxelError<T, S> where T : VoxelCoord, S : VoxelCoord {
    NotLoaded(VoxelPos<T>, VoxelPos<T>),
    ChunkBoundsInvalid(VoxelPos<T>, VoxelPos<T>, VoxelSize<S>, VoxelSize<S>, VoxelRange<T>),
}
impl<T, S> fmt::Display for ChunkedVoxelError<T, S> where T : VoxelCoord, S : VoxelCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChunkedVoxelError::NotLoaded(blockpos, pos) => write!(f, "Chunk at {} not yet loaded, cannot access block {}", pos, blockpos),
            ChunkedVoxelError::ChunkBoundsInvalid(blockpos, chunkpos, expectedchunksize, actualchunksize, actualbounds) => write!(f, 
                                "Failed attempt to access block {}: Chunk size invalid. Chunk at {} is supposed to be of size {}, and it is {}. Its bounds are {}.", 
                                blockpos, chunkpos, expectedchunksize, actualchunksize, actualbounds),
        }
    }
}
impl<T, S> Error for ChunkedVoxelError<T, S> where T : VoxelCoord, S : VoxelCoord {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

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
    vpos!((point.x as f32 / chunk_size.x as f32).floor() as i32, 
        (point.y as f32 / chunk_size.y as f32).floor() as i32, 
        (point.z as f32 / chunk_size.z as f32).floor() as i32)
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

#[test]
fn test_chunkpos() { 
    assert!(blockpos_to_chunk(vpos!(6, -1, 7), vpos!(16, 16, 16)) == vpos!(0, -1, 0));
    assert!(blockpos_to_chunk(vpos!(17, -25, 2), vpos!(8, 24, 4)) == vpos!(2, -2, 0));
}

impl VoxelStorage<BlockID, i32> for Dimension {
    fn get(&self, coord: VoxelPos<i32>) -> Result<BlockID, Box<Error>>{
        let size = self.chunk_size.clone();
        let chunkpos = blockpos_to_chunk(coord, size);
        // Do we have a chunk that would contain this block position?
        match self.chunks.get(&chunkpos) {
            Some(chunk_entry_arc) => {
                let chunk_entry = chunk_entry_arc.clone();
                let bounds = chunk_entry.bounds.clone();
                let chunk_size = bounds.get_size_unsigned();
                if chunk_size != size {
                    return Err(Box::new(ChunkedVoxelError::ChunkBoundsInvalid(coord, chunkpos, size, chunk_size, bounds)));
                }
                match bounds.get_local_unsigned(coord) {
                    Some(pos) => {
                        // Block until we can get a valid voxel.
                        let locked = chunk_entry.data.read();
                        return Ok(locked.get(vpos!(pos.x as u8, pos.y as u8, pos.z as u8))?);
                    },
                    // Position is not inside our chunk's bounds.
                    None => return Err(Box::new(ChunkedVoxelError::<i32, u32>::ChunkBoundsInvalid(coord, chunkpos, size, chunk_size, bounds))),
                }
            },
            // Chunk not currently loaded or generated.
            None => return Err(Box::new(ChunkedVoxelError::<i32, u32>::NotLoaded(chunkpos,coord))),
        }
    }
    fn set(&mut self, coord: VoxelPos<i32>, value: BlockID) -> Result<(), Box<Error>>{
        let size = self.chunk_size.clone();
        // Do we have a chunk that would contain this block position?
        let chunkpos = blockpos_to_chunk(coord, size);
        let rslt = self.chunks.get(&chunkpos).cloned();
        match rslt {
            Some(chunk_entry) => {
                let bounds = chunk_entry.bounds.clone();
                let chunk_size = bounds.get_size_unsigned();
                if chunk_size != size {
                    return Err(Box::new(ChunkedVoxelError::ChunkBoundsInvalid(coord, chunkpos, size, chunk_size, bounds)));
                }
                match bounds.get_local_unsigned(coord) {
                    Some(pos) => {
                        // Block until we can write.
                        let mut locked = chunk_entry.data.write();
                        let position = vpos!(pos.x as u8, pos.y as u8, pos.z as u8);
                        let current = locked.get(position)?;
                        if current != value {
                            chunk_entry.state.store(CHUNK_STATE_DIRTY, Ordering::Relaxed); //Mark for remesh.
                            locked.set(position, value)?;
                        }
                    },
                    // Position is not inside our chunk's bounds.
                    None => return Err(Box::new(ChunkedVoxelError::<i32, u32>::ChunkBoundsInvalid(coord, chunkpos, size, chunk_size, bounds))),
                }
            },
            // Chunk not currently loaded or generated.
            None => return Err(Box::new(ChunkedVoxelError::<i32, u32>::NotLoaded(chunkpos,coord))),
        }
        Ok(())
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