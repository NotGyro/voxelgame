//extern crate serde;
extern crate std;
extern crate num;

use std::error::Error;
use std::fmt::Debug;
use std::result::Result;

use self::num::Integer;
use voxel::*;
use voxel::voxelmath::*;
use voxel::voxelstorage::VoxelStorage;
use voxel::voxelarray::VoxelArray;

pub type EventTypeID = u8;

/*
#[derive(Debug, Clone)]
struct EventApplyError {}

impl Error for EventApplyError {
    fn description(&self) -> &str {
        "An attempt to apply a VoxelEvent to a VoxelStorage has failed."
    }
}
*/

pub type EventApplyResult = Result<(), Box<Error>>;

/// Represents a change to the contents of a Voxel Storage.
/// Type arguments are voxel type, position type. This is the version of this trait
/// with no run-time type information.
pub trait VoxelEventUntyped<T, P> : Clone where T : Clone, P : Copy + Integer{
    /// Applies a voxel event to a VoxelStorage.
    /// The intended use of this is as a default case, and ideally specific 
    /// VoxelStorage implementations could provide better-optimized 
    fn apply_blind(&self, stor : &mut VoxelStorage<T, P>) -> EventApplyResult;
}

/// Type arguments are voxel type, position type.
pub trait VoxelEvent<T, P>: VoxelEventUntyped<T, P> where T : Clone, P : Copy + Integer {
    const TYPE_ID: EventTypeID;
    fn get_type_id() -> EventTypeID { Self::TYPE_ID }
}

// ---- Actual event structs and their VoxelEventUntyped implementations. ----

#[derive(Clone, Debug)]
pub struct OneVoxelChange<T : Clone, P : Copy + Integer> {
    new_value : T,
    pos : VoxelPos<P>,
}

#[derive(Clone, Debug)]
pub struct SetVoxelRange<T : Clone, P : Copy + Integer> { 
    new_value : T, 
    range : VoxelRange<P>,
}

impl <T, P> VoxelEventUntyped<T, P> for OneVoxelChange<T, P> where T : Clone, P : Copy + Integer {
    fn apply_blind(&self, stor : &mut VoxelStorage<T, P>) -> EventApplyResult {
        stor.set(self.pos, self.new_value.clone());
        Ok(()) // TODO: modify VoxelStorage's "Set" method to return errors rather than silently fail
    }
}

impl <T, P> VoxelEventUntyped<T, P> for SetVoxelRange<T, P> where T : Clone, P : Copy + Integer {
    fn apply_blind(&self, stor : &mut VoxelStorage<T, P>) -> EventApplyResult {
        for pos in self.range {
            stor.set(pos, self.new_value.clone()); 
        }
        Ok(()) // TODO: modify VoxelStorage's "Set" method to return errors rather than silently fail
    }
}

// ----------------------- Tests -----------------------

// Used for tests
const CHUNK_X_LENGTH : u32 = 16;
const CHUNK_Y_LENGTH : u32 = 16;
const CHUNK_Z_LENGTH : u32 = 16;
const OURSIZE : usize = (CHUNK_X_LENGTH * CHUNK_Y_LENGTH * CHUNK_Z_LENGTH) as usize;

#[test]
fn test_apply_voxel_event() { 
    let mut array : Vec<String> = vec!["Hello!".to_string(); OURSIZE];
    let mut storage : VoxelArray<String, u32> = VoxelArray::load_new(CHUNK_X_LENGTH, CHUNK_Y_LENGTH, CHUNK_Z_LENGTH, array);
    let evt : OneVoxelChange<String, u32> = OneVoxelChange{ new_value : "World!".to_string(), pos : VoxelPos { x: 7, y: 7, z:7}}; 
    evt.apply_blind(&mut storage).unwrap();
    assert_eq!(storage.get(VoxelPos{x: 6, y: 6, z: 6} ).unwrap(), "Hello!".to_string());
    assert_eq!(storage.get(VoxelPos{x: 7, y: 7, z: 7} ).unwrap(), "World!".to_string());
}