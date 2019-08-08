//! A dimension.


use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicUsize;

use std::collections::HashMap;
use cgmath::{Point3, MetricSpace};
use renderer::LineRenderQueue;
use world::generators::{WorldGenerator, PerlinGenerator};
use voxel::voxelstorage::*;
use voxel::voxelarray::*;
use voxel::voxelmath::*;

// TODO: Rewrite this to use the standard Futures API.
/// State used for multithreaded chunk loading. Chunk is dirty and needs to be generated.
pub static CHUNK_STATE_DIRTY: usize = 0;
/// State used for multithreaded chunk loading. Chunk is currently being generated.
pub static CHUNK_STATE_WRITING: usize = 1;
/// State used for multithreaded chunk loading. Chunk is finished being generated.
pub static CHUNK_STATE_CLEAN: usize = 2;

/// A dimension.
pub struct Dimension {
    pub chunks: HashMap<(i32, i32, i32), (Arc<RwLock<VoxelArray<u8, u8>>>, Arc<AtomicUsize>)>,
    pub chunk_size: (i32, i32, i32),
}


impl Dimension {
    pub fn new() -> Dimension {
        Dimension {
            chunks: HashMap::new(),
            chunk_size: (16, 16, 16),
        }
    }
    pub fn chunkpos_to_block(&self, point: (i32, i32, i32) ) -> (i32, i32, i32) { 
        (point.0 * self.chunk_size.0 as i32, 
                    point.1 * self.chunk_size.1 as i32, 
                    point.2 * self.chunk_size.2 as i32)
    }
    pub fn chunkpos_to_center(&self, point: (i32, i32, i32) ) -> Point3<f32> { 
        let block_pos = self.chunkpos_to_block(point);
        Point3::new(block_pos.0 as f32 + (self.chunk_size.0 as f32 * 0.5), 
                    block_pos.1 as f32 + (self.chunk_size.1 as f32 * 0.5), 
                    block_pos.2 as f32 + (self.chunk_size.2 as f32 * 0.5))
    }

    pub fn is_chunk_loaded(&self, chunk_pos : (i32, i32, i32) ) -> bool {self.chunks.contains_key(&chunk_pos)}

    pub fn loaded_chunk_list(&self) -> Vec<(i32, i32, i32)> {
        let mut result = Vec::new();
        for pos in self.chunks.keys() {
            result.push(*pos);
        }
        result
    }

    /// Adds new chunks as the player moves closer to them, and removes old chunks as the player
    /// moves away.
    pub fn load_unload_chunks(&mut self, player_pos: Point3<f32>, queue: &mut LineRenderQueue) {
        const CHUNK_RADIUS: i32 = 2;
        const CHUNK_DISTANCE: f32 = CHUNK_RADIUS as f32 * 2.0 * 16.0;
        const RETAIN_RADIUS: f32 = CHUNK_DISTANCE + 4.0; // offset added to prevent load/unload loop on the edge

        let chunk_size = self.chunk_size.clone();

        let gen = PerlinGenerator::new();
        
        self.chunks.retain(|pos, _| {
            let block_pos = (pos.0 * chunk_size.0 as i32, 
                    pos.1 * chunk_size.1 as i32, 
                    pos.2 * chunk_size.2 as i32);
            let chunk_pos = Point3::new(block_pos.0 as f32 + (chunk_size.0 as f32 * 0.5), 
                        block_pos.1 as f32 + (chunk_size.1 as f32 * 0.5), 
                        block_pos.2 as f32 + (chunk_size.2 as f32 * 0.5));
            let dist = Point3::distance(chunk_pos, player_pos);
            dist < RETAIN_RADIUS // offset added to prevent load/unload loop on the edge
        });

        let player_x_in_chunks = (player_pos.x / (self.chunk_size.0 as f32)) as i32;
        let player_y_in_chunks = (player_pos.y / (self.chunk_size.1 as f32)) as i32;
        let player_z_in_chunks = (player_pos.z / (self.chunk_size.2 as f32)) as i32;
        for cx in (player_x_in_chunks-CHUNK_RADIUS)..(player_x_in_chunks+CHUNK_RADIUS+1) {
            for cy in (player_y_in_chunks-CHUNK_RADIUS)..(player_y_in_chunks+CHUNK_RADIUS+1) {
                for cz in (player_z_in_chunks-CHUNK_RADIUS)..(player_z_in_chunks+CHUNK_RADIUS+1) {
                    let chunk_pos = (cx as i32, cy as i32, cz as i32);
                    if self.chunks.contains_key(&chunk_pos) {
                        continue;
                    }

                    let chunk_world_pos = self.chunkpos_to_center((cx, cy, cz));
                    let dist = Point3::distance(chunk_world_pos, player_pos);
                    if dist < CHUNK_DISTANCE {
                        let chunk_origin = self.chunkpos_to_block((cx, cy, cz));
                        let mut range = VoxelRange{lower: VoxelPos::from(chunk_origin), 
                                        upper : VoxelPos::from(chunk_origin) + VoxelPos::from(self.chunk_size)};
                        range.validate();
                        let mut chunk = gen.generate(range, 0);
                        self.chunks.insert(chunk_pos, (Arc::new(RwLock::new(chunk)), Arc::new(AtomicUsize::new(CHUNK_STATE_DIRTY))));
                        queue.chunks_changed = true;
                    }
                }
            }
        } /*
        //Let's debug our renderer!
        if ! (self.chunks.contains_key(&(0,0,0))) {
            let mut range = VoxelRange{lower: vpos!(0,0,0), 
                                        upper : VoxelPos::from(self.chunk_size)};
            range.validate();
            let mut chunk = gen.generate(range, 0);
            self.chunks.insert((0,0,0), (Arc::new(RwLock::new(chunk)), Arc::new(AtomicUsize::new(CHUNK_STATE_DIRTY))));
            queue.chunks_changed = true;
        }*/
    }
}