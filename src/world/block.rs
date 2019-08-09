extern crate string_cache;
extern crate parking_lot;

use self::string_cache::DefaultAtom as Atom;
use voxel::voxelmath::*;
use std::collections::HashMap;

use self::parking_lot::Mutex;
use voxel::voxelarray::VoxelArray;

pub type BlockID = u32;
pub type BlockName = Atom;
pub type Chunk = VoxelArray<BlockID, u8>;

pub struct BlockRegistry {
    id_to_name : Vec<BlockName>,
    name_to_id : HashMap<BlockName,BlockID>,
}

impl BlockRegistry {
    pub fn id_for_name(&self, id : &BlockID) -> BlockName{
        self.id_to_name.get(*id as usize).unwrap().clone()
    }
    pub fn name_for_id(&self, name : &BlockName) -> BlockID{ self.name_to_id.get(name).unwrap().clone() }
    pub fn all_mappings(&self) -> HashMap<BlockName, BlockID> { self.name_to_id.clone()}
    pub fn register_block(&mut self, name: &BlockName) -> BlockID { 
        {
            assert!(self.name_to_id.contains_key(name) == false);
        }
        let new_id = self.id_to_name.len() as BlockID;
        self.id_to_name.push(name.clone());
        self.name_to_id.insert(name.clone(), new_id.clone());
        return new_id;
    }
}

lazy_static! {
    pub static ref MASTER_BLOCK_REGISTRY : Mutex<BlockRegistry> = {
        Mutex::new(BlockRegistry { 
            id_to_name : Vec::new(),
            name_to_id : HashMap::new(),
        })
    };
}
