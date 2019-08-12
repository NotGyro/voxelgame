extern crate std;
extern crate num;

use voxel::voxelmath::VoxelCoord;
use voxel::voxelmath::VoxelPos;
use voxel::voxelmath::VoxelRange;
use std::fmt::Debug;
use voxel::voxelevent::VoxelEvent;
use voxel::voxelevent::VoxelEventInner;
use std::error;
use std::result::Result;

pub trait Voxel : Clone + Debug {}
impl<T> Voxel for T where T : Clone + Debug {}

/// A basic trait for any 3d grid data structure.
/// Type arguments are type of element, type of position.
///
/// (Type of positon must be an integer, but I'm still using
/// genericism here because it should be possible to use 
/// any bit length of integer, or even a bigint implementation
///
/// For this trait, a single level of detail is assumed.
///
/// For voxel data structures with a level of detail, we will
/// assume that the level of detail is a signed integer, and
/// calling these methods / treating them as "flat" voxel
/// structures implies acting on a level of detail of 0.

pub trait VoxelStorage<T: Voxel, P: VoxelCoord> {
    // Get and Set are all you need to implement a Voxel Storage.
    fn get(&self, coord: VoxelPos<P>)  -> Result<T, Box<error::Error>>;
    fn set(&mut self, coord: VoxelPos<P>, value: T) -> Result<(), Box<error::Error>>;

    fn apply_event(&mut self, e : VoxelEvent<T, P>) -> Result<(), Box<error::Error>> where Self: std::marker::Sized {
        e.apply_blind(self)?;
        Ok(())
    }
}
/*
pub trait VoxelStorageIOAble<T : Clone, P: Copy + Integer> : VoxelStorage<T, P> where P : Copy + Integer {
    fn load<R: Read + Sized>(&mut self, reader: &mut R);
    fn save<W: Write + Sized>(&self, writer: &mut W) -> Result<usize, std::io::Error>;
}
*/

/// Any VoxelStorage which has defined, finite bounds.
/// Must provide a valid voxel for any position within
/// the range provided by get_bounds().
/// Usually, this implies that the voxel storage is not paged.
pub trait VoxelStorageBounded<T: Voxel, P: VoxelCoord> : VoxelStorage<T, P> { 
    fn get_bounds(&self) -> VoxelRange<P>;
}

/// Copy voxels from one storage to another. 
pub fn voxel_blit<T: Voxel, P: VoxelCoord>(source_range : VoxelRange<P>, source: &VoxelStorage<T, P>, 
                                                dest_origin: VoxelPos<P>, dest: &mut VoxelStorage<T,P>)  -> Result<(), Box<error::Error>> {
    for pos in source_range {
        let voxel = source.get(pos)?;
        let offset_pos = (pos - source_range.lower) + dest_origin;
        dest.set(offset_pos, voxel)?;
    }
    return Ok(());
}