use noise::{NoiseFn, Perlin, Seedable};
use super::WorldGenerator;
use ::world::Chunk;


pub struct PerlinGenerator {
    perlin: Perlin,
    scale: f64,
    offset: f64
}


impl PerlinGenerator {
    pub fn new() -> PerlinGenerator {
        let perlin = Perlin::new();
        perlin.set_seed(1);
        PerlinGenerator {
            perlin,
            scale: 0.0102,
            offset: 0.26378
        }
    }
}


impl WorldGenerator for PerlinGenerator {
    fn generate(&self, pos: (i32, i32, i32), dimension_id: u32) -> Chunk {
        let mut chunk = Chunk::new(pos, dimension_id);
        let mut data = [0u8; 16*16*16];
        for x in 0..16 {
            for z in 0..16 {
                let height_norm = self.perlin.get([(pos.0*16 + x) as f64 * self.scale + self.offset, (pos.2*16 + z) as f64 * self.scale + self.offset]) / 2.0 + 0.5;
                let height_abs = height_norm as f32 * 16.0;
                for y in 0..16 {
                    if (pos.1 as f32 * 16.0) + y as f32 <= height_abs {
                        data[Chunk::xyz_to_i(x, y, z)] =  2u8;
                    }
                }
            }
        }
        chunk.replace_data(&data);

        chunk
    }
}