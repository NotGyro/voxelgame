extern crate std;
extern crate num;

use std::marker::Copy;

use voxel::voxelstorage::*;
use voxel::voxelmath::*;
use std::io::prelude::*;
use std::mem;
use std::default::Default;

use self::num::Integer;

/// A 3D packed array of voxels - it's a single flat buffer in memory,
/// which is indexed by voxel positions with some math done on them. 
/// Should have a fixed, constant size after creation.
#[derive(Clone, Debug)]
pub struct VoxelArray<T: Clone, P: Copy + Integer + USizeAble> {
    size_x: P, size_y: P, size_z: P,
    data: Vec<T>,
}

pub fn xyz_to_i<P: Copy + Integer + USizeAble>(x : P, y : P, z : P, size_x: P, size_y: P, size_z: P) -> usize {
    ((z.as_usize() * (size_x.as_usize() * size_y.as_usize())) + (y.as_usize() * (size_x.as_usize())) + x.as_usize())
}

impl <T:Clone, P: Copy + Integer + USizeAble> VoxelArray<T, P> {
    pub fn load_new(szx: P, szy: P, szz: P, dat: Vec<T>) -> VoxelArray<T, P> {
        VoxelArray{size_x: szx, size_y: szy, size_z: szz, data: dat}
    }

    /// Make a new VoxelArray wherein every value is set to val
    pub fn new_solid(szx: P, szy: P, szz: P, val:T) -> VoxelArray<T, P> {
        VoxelArray{size_x: szx, size_y: szy, size_z: szz, data: vec![ val; (szx*szy*szz).as_usize()] }
    }

    /// Replaces the data inside a chunk all at once. This drops the old self.data.
    pub fn replace_data(&mut self, data: Vec<T>) {
        // TODO: Better error handling here 
        // Make sure these are the same size and not going to invalidate our size fields.
        assert_eq!(self.data.len(), data.len());
        self.data = data;
    }
}

impl <T:Clone + Default, P: Copy + Integer + USizeAble> VoxelArray<T, P> {
    /// Make a new VoxelArray wherein every value is set to T::Default
    pub fn new_empty(szx: P, szy: P, szz: P) -> VoxelArray<T, P> { VoxelArray::new_solid(szx, szy, szz,T::default()) }
}

impl <T: Clone, P: Copy + Integer + USizeAble> VoxelStorage<T, P> for VoxelArray<T, P> {
    fn get(&self, coord: VoxelPos<P>) -> Option<T> {
    	//Bounds-check.
    	if (coord.x >= self.size_x) ||
    		(coord.y >= self.size_y) ||
    		(coord.z >= self.size_z)
    	{
    		return None;
    	}
    	//Packed array access
    	return self.data.get(xyz_to_i(coord.x, coord.y, coord.z, self.size_x, self.size_y, self.size_z)).map(|v| v.clone());
    }

    fn set(&mut self, coord: VoxelPos<P>, value: T) {
    	if (coord.x >= self.size_x) ||
    		(coord.y >= self.size_y) ||
    		(coord.z >= self.size_z)
    	{
    		return;
    	}
    	//Packed array access
    	(*self.data.get_mut(xyz_to_i(coord.x, coord.y, coord.z, self.size_x, self.size_y, self.size_z)).unwrap()) = value;
    }
}

/*
impl <T: Clone, P> VoxelStorageIOAble<T, P> for VoxelArray<T, P> where P : Copy + Integer + USizeAble { 
    #[allow(mutable_transmutes)]
    #[allow(unused_must_use)]
    fn load<R: Read + Sized>(&mut self, reader: &mut R) { 
		let array: &mut [u8] = unsafe { mem::transmute(&*self.data) };
    	reader.read(array);
    }
    
    #[allow(mutable_transmutes)]
    #[allow(unused_must_use)]
    fn save<W: Write + Sized>(&self, writer: &mut W) -> Result<usize, std::io::Error> {
		let array: &[u8] = unsafe { mem::transmute(&*self.data) };
    	writer.write(array)
    }
}*/

impl <T: Clone, P> VoxelStorageBounded<T, P> for VoxelArray<T, P> where P : Copy + Integer + USizeAble { 
    fn get_bounds(&self) -> VoxelRange<P> { VoxelRange {lower: VoxelPos{x: P::zero(),y: P::zero(), z:P::zero()},  
                                            upper: VoxelPos{x: self.size_x, y: self.size_y, z: self.size_z} } }
}

#[test]
fn test_array_raccess() {
    const OURSIZE : usize  = 16 * 16 * 16;
    let mut test_chunk : Vec<u16> = Vec::with_capacity(OURSIZE);
    for i in 0 .. OURSIZE {
    	test_chunk.push(i as u16);
    }

    let mut test_va : VoxelArray<u16,u16> = VoxelArray::load_new(16, 16, 16, test_chunk);
    
    let testpos = VoxelPos{x: 14, y: 14, z: 14};
    assert!(test_va.get(testpos).unwrap() == 3822);
    test_va.set(testpos,9);
    assert!(test_va.get(testpos).unwrap() == 9);
}


#[test]
fn test_array_iterative() {
    const OURSIZE : usize  = 16 * 16 * 16;
    let mut test_chunk : Vec<u16> = Vec::with_capacity(OURSIZE);
    for _i in 0 .. OURSIZE {
    	test_chunk.push(16);
    }

    let mut test_va : VoxelArray<u16, u16> = VoxelArray::load_new(16, 16, 16, test_chunk);
    for pos in test_va.get_bounds() {
    	assert!(test_va.get(pos).unwrap() == 16);
    	test_va.set(pos, (pos.x as u16 % 10));
    }
    assert!(test_va.get(VoxelPos{x: 10, y: 0, z: 0}).unwrap() == 0);
    assert!(test_va.get(VoxelPos{x: 11, y: 0, z: 0}).unwrap() == 1);
    //assert_eq!(test_va.get_data_size(), (OURSIZE * 2));
}
