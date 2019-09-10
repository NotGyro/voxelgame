extern crate std;
extern crate num;

use voxel::voxelmath::{VoxelCoord, VoxelPos, VoxelRange};
use std::fmt::{Display, Debug};
use std::fmt;
use voxel::voxelevent::{VoxelEvent, VoxelEventInner};
use std::error;
use std::error::Error;
use std::result::Result;

pub trait Voxel : Clone + Debug {}
impl<T> Voxel for T where T : Clone + Debug {}

pub enum VoxelErrorKind {
    OutOfBounds,
    NotYetLoaded,
    SetInvalidValue,
    InvalidValueAt,
    Other,
}
/// An error reported upon trying to get or set a voxel outside of our range. 
#[derive(Debug)]
pub enum VoxelError {
    OutOfBounds(String, String),
    NotYetLoaded(String),
    SetInvalidValue(String),
    InvalidValueAt(String),
    Other(Box<dyn error::Error + 'static>),
}

impl VoxelError { 
    fn kind(&self) -> VoxelErrorKind {
        match self { 
            VoxelError::OutOfBounds(_,_) => VoxelErrorKind::OutOfBounds,
            VoxelError::NotYetLoaded(_) => VoxelErrorKind::NotYetLoaded,
            VoxelError::SetInvalidValue(_) => VoxelErrorKind::SetInvalidValue,
            VoxelError::InvalidValueAt(_) => VoxelErrorKind::InvalidValueAt,
            VoxelError::Other(_) => VoxelErrorKind::Other,
        }
    }
}

impl Display for VoxelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self { 
            VoxelError::OutOfBounds(pos,sz) => write!(f, "Attempted to access a voxel at position {} on a storage with bounds {}", pos, sz),
            VoxelError::NotYetLoaded(pos) => write!(f, "Attempted to access a voxel position {}, which is not yet loaded.", pos),
            VoxelError::SetInvalidValue(pos) => write!(f, "Attempted to set voxel at {} to an invalid value.", pos),
            VoxelError::InvalidValueAt(pos) => write!(f, "Voxel at {} contains an invalid value, most likely corrupt.", pos),
            VoxelError::Other(err) => write!(f, "Other voxel error: {}", err),
        }
    }
}
impl Error for VoxelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None //I would love to have it to handle Other correctly but nope, the sized variablre requirement isn't having it.
    }
}

/*impl<T> From<Box<dyn error::Error + 'static>> for VoxelError<T> where T : 'static + VoxelCoord{
    fn from(error: Box<dyn error::Error + 'static>) -> Self {
        VoxelError::Other(error)
    }
}*/ /*
impl From<Box<dyn error::Error + 'static>> for VoxelError {
    fn from(error: Box<dyn error::Error + 'static>) -> Self {
        VoxelError::Other(error)
    }
}*/

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
    fn get(&self, coord: VoxelPos<P>) -> Result<T, VoxelError>;
    fn set(&mut self, coord: VoxelPos<P>, value: T) -> Result<(), VoxelError>;

    fn apply_event(&mut self, e : VoxelEvent<T, P>) -> Result<(), VoxelError> where Self: std::marker::Sized {
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
pub fn voxel_blit<T: Voxel, P: VoxelCoord>(source_range : VoxelRange<P>, source: &dyn VoxelStorage<T, P>, 
                                                dest_origin: VoxelPos<P>, dest: &mut dyn VoxelStorage<T,P>)  -> Result<(), VoxelError> {
    for pos in source_range {
        let voxel = source.get(pos)?;
        let offset_pos = (pos - source_range.lower) + dest_origin;
        dest.set(offset_pos, voxel)?;
    }
    return Ok(());
}