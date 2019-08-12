//! Simple world generator using perlin noise.


use noise::{NoiseFn, Perlin, Seedable};
use world::generators::WorldGenerator;

use voxel::voxelmath::*;
use voxel::voxelarray::*;
use world::block::{BlockID, Chunk};

/// Simple world generator using perlin noise.
pub struct PerlinGenerator {
    perlin: Perlin,
    scale: f64,
    offset: f64,
    block_type_noise: Perlin,
    block_type_scale: f64,
}


impl PerlinGenerator {
    /// Creates a new `PerlinGenerator`
    pub fn new() -> PerlinGenerator {
        let perlin = Perlin::new();
        perlin.set_seed(1);

        let block_type_noise = Perlin::new();
        perlin.set_seed(50);

        PerlinGenerator {
            perlin,
            scale: 0.008126,
            offset: 0.26378,
            block_type_noise,
            block_type_scale: 0.063647,
        }
    }
}


impl WorldGenerator for PerlinGenerator {
    fn generate(&self, bounds: VoxelRange<i32>, _dimension_id: u32) -> Chunk {
        let size = bounds.get_size();
        
        let num_elements = (size.x * size.y * size.z) as usize;
        let mut data : Vec<BlockID> = Vec::with_capacity(num_elements);
        for _ in 0..num_elements { data.push(0); }

        for x in 0..size.x {
            for z in 0..size.z {
                let height_norm = self.perlin.get([((bounds.lower.x + x) as f64 + self.offset) * self.scale, 
                                                    ((bounds.lower.z + z) as f64 + self.offset) * self.scale]) / 2.0 + 0.5;
                let height_abs = height_norm as f32 * (size.y * 2) as f32;
                for y in 0..size.y {
                    if (bounds.lower.y + y) as f32 <= height_abs {
                        let block_type_val = self.block_type_noise.get([((bounds.lower.x + x) as f64) * self.block_type_scale, 
                                                                        ((bounds.lower.z + z) as f64) * self.block_type_scale]) / 2.0 + 0.5;
                        let block_id = ((block_type_val * 3.0) + 1.0) as BlockID;

                        data[xyz_to_i(x as usize, y as usize, z as usize, 
                                        size.x as usize, size.y as usize, size.z as usize)] = block_id;
                    }
                }
            }
        }
        VoxelArray::load_new(size.x as u8, size.y as u8, size.z as u8, data)
    }
}