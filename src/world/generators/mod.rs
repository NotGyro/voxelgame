//! World generator types.

pub mod perlingenerator;

pub use self::perlingenerator::PerlinGenerator;

use voxel::voxelmath::*;
use voxel::voxelarray::*;

use world::block::{BlockID, Chunk};

/// Trait for world generators.
pub trait WorldGenerator {
    /// Generates a chunk with this generator.
    fn generate(&self, bounds: VoxelRange<i32>, dimension_id: u32) -> Chunk;
}