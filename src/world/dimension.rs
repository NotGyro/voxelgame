use std::collections::HashMap;
use cgmath::{Point3, MetricSpace};
use ::world::Chunk;
use ::world::generators::{WorldGenerator, PerlinGenerator};


pub struct Dimension {
    pub chunks: HashMap<(i32, i32, i32), Chunk>
}


impl Dimension {
    pub fn new() -> Dimension {
        Dimension {
            chunks: HashMap::new(),
        }
    }


    pub fn load_unload_chunks(&mut self, player_pos: Point3<f32>) {
        const CHUNK_DISTANCE: f32 = 8.0 * 16.0;
        self.chunks.retain(|pos, _| {
            let chunk_pos = Point3::new(pos.0 as f32 * 16.0 + 8.0, pos.1 as f32 * 16.0 + 8.0, pos.2 as f32 * 16.0 + 8.0);
            let dist = Point3::distance(chunk_pos, player_pos);
            dist < CHUNK_DISTANCE + 4.0 // offset added to prevent load/unload loop on the edge
        });

        let gen = PerlinGenerator::new();
        let player_x_in_chunks = (player_pos.x / 16.0) as i32;
        let player_z_in_chunks = (player_pos.z / 16.0) as i32;
        for cx in (player_x_in_chunks-4)..(player_x_in_chunks+5) {
            for cz in (player_z_in_chunks-4)..(player_z_in_chunks+5) {
                let chunk_pos = (cx as i32, 0i32, cz as i32);
                if self.chunks.contains_key(&chunk_pos) {
                    continue;
                }

                let chunk_world_pos = Point3::new(cx as f32 * 16.0 + 8.0,
                                                  0.0,
                                                  cz as f32 * 16.0 + 8.0);
                let dist = Point3::distance(chunk_world_pos, player_pos);
                if dist < CHUNK_DISTANCE {
                    println!("adding chunk @ {:?}", chunk_pos);
                    let mut chunk = gen.generate(chunk_pos, 0);
                    chunk.mesh_dirty = true;
                    self.chunks.insert(chunk_pos, chunk);
                }
            }
        }
    }
}