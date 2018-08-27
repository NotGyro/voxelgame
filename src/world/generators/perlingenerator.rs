use noise::{NoiseFn, Perlin, Seedable};
use super::WorldGenerator;
use ::world::Chunk;


pub struct PerlinGenerator {
    perlin: Perlin,
    scale: f64
}


impl PerlinGenerator {
    pub fn new() -> PerlinGenerator {
        let perlin = Perlin::new();
        perlin.set_seed(1);
        PerlinGenerator {
            perlin,
            scale: 0.0102
        }
    }
}


impl WorldGenerator for PerlinGenerator {
    fn generate(&self, pos: (i32, i32, i32), dimension_id: u32) -> Chunk {
        let mut chunk = Chunk::new(pos, dimension_id);
        for x in 0..16 {
            for z in 0..16 {
                let height = self.perlin.get([(pos.0*16 + x) as f64 * self.scale, (pos.2*16 + z) as f64 * self.scale]) / 2.0 + 0.5;
                for y in 0..((height * 16.0) as i32) {
                    chunk.set_at(Chunk::xyz_to_i(x, y, z), 2u8);
                }
            }
        }
        chunk
    }
}