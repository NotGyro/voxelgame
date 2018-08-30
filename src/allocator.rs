extern crate fine_grained;

use std::collections::HashMap;
use std::ops::Range;


// TODO: tests


#[derive(Debug, Eq, Hash, Clone)]
pub struct BlockId(pub usize);

impl PartialEq for BlockId {
    fn eq(&self, other: &BlockId) -> bool {
        self.0 == other.0
    }
}


#[derive(Debug)]
pub struct BlockAllocator {
    pub size: usize,
    pub allocs: HashMap<BlockId, Range<usize>>
}


impl BlockAllocator {
    pub fn new(size: usize) -> BlockAllocator {
        BlockAllocator {
            size,
            allocs: HashMap::new()
        }
    }


    pub fn get_next_free_id(&self) -> BlockId {
        let mut id = BlockId(1);
        while self.allocs.contains_key(&id) {
            id.0 += 1;
        }
        id
    }


    /// returns (BlockPtr, offset)
    pub fn alloc(&mut self, size: usize, alignment: usize) -> Option<(BlockId, usize)> {
        let mut block_ends = vec![0];
        for (_, range) in self.allocs.iter() {
            let mut e = range.end;
            // skip bytes until aligned
            if alignment != 0 {
                while e % alignment != 0 {
                    e += 1;
                }
            }
            block_ends.push(e);
        }
        let mut block_starts = vec![self.size];
        for (_, range) in self.allocs.iter() {
            block_starts.push(range.start);
        }

        'outer: for end in block_ends.iter() {
            'inner: for start in block_starts.iter() {
                if (*start as i32 - *end as i32) < 0i32 {
                    // start is before end, skip
                    continue 'inner;
                }
                if start - end < size {
                    // found a start too close after current end, gap not big enough
                    continue 'outer;
                }
            }
            // no start too close after current end, gap big enough
            let next_id = self.get_next_free_id();
            self.allocs.insert(next_id.clone(), *end..(*end+size));
            return Some((next_id, *end));
        }
        // couldn't find any gaps
        None
    }


    pub fn free(&mut self, ptr: &BlockId) {
        self.allocs.remove(ptr);
    }
}