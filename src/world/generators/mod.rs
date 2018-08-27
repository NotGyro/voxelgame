mod perlingenerator;

pub use self::perlingenerator::PerlinGenerator;


pub trait WorldGenerator {
    fn generate(&self, pos: (i32, i32, i32), dimension_id: u32) -> super::Chunk;
}