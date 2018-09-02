//! Memory allocator types.
//!
//! [BlockAllocator] is a virtual block allocator. It doesn't manage actual memory, only virtual allocations.
//!
//! [PoolAllocator] is a physical device memory allocator. Used by [AutoMemoryPool](::memory::pool::AutoMemoryPool).

use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, RwLock};

use vulkano::memory::pool::StdHostVisibleMemoryTypePool;

use super::pool::{AutoMemoryPoolChunk, AutoMemoryPoolBlock, AutoMemoryPoolInner, AUTO_POOL_CHUNK_SIZE};


// TODO: tests


/// ID corresponding to an allocated block.
#[derive(Debug, Eq, Hash, Clone)]
pub struct BlockId(pub usize);

impl PartialEq for BlockId {
    fn eq(&self, other: &BlockId) -> bool {
        self.0 == other.0
    }
}


/// Virtual block allocator.
///
/// It doesn't actually manage any memory, it just keeps track of which regions of some area are
/// allocated by something. Used by [AutoMemoryPool](::memory::pool::AutoMemoryPool) to keep track
/// of which areas of a chunk have been allocated.
#[derive(Debug)]
pub struct BlockAllocator {
    pub size: usize,
    pub allocs: HashMap<BlockId, Range<usize>>
}


impl BlockAllocator {
    /// Creates a new BlockAllocator to manage the given size. Since BlockAllocator doesn't actually
    /// manage memory, "size" is in whatever units the user wants.
    pub fn new(size: usize) -> BlockAllocator {
        BlockAllocator {
            size,
            allocs: HashMap::new()
        }
    }


    /// Returns the first unused block ID.
    pub fn get_first_free_id(&self) -> BlockId {
        let mut id = BlockId(1);
        while self.allocs.contains_key(&id) {
            id.0 += 1;
        }
        id
    }


    /// Allocates a new region and returns `Some((BlockId, offset))`, or `None` if it couldn't find
    /// a free space big enough.
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
            let next_id = self.get_first_free_id();
            self.allocs.insert(next_id.clone(), *end..(*end+size));
            return Some((next_id, *end));
        }
        // couldn't find any gaps
        None
    }


    /// Frees the block with the given id.
    pub fn free(&mut self, ptr: &BlockId) {
        self.allocs.remove(ptr);
    }
}


/// Allocator that manages a pool of device memory for a certain memory type. It handles allocating
/// new chunks of device memory as necessary, and providing allocated blocks from a chunk when
/// requested.
///
/// [AutoMemoryPoolBlock.drop](::memory::pool::AutoMemoryPoolBlock) handles freeing that block
/// from its chunk.
#[derive(Debug)]
pub struct PoolAllocator {
    pub pool: Arc<StdHostVisibleMemoryTypePool>,
    pub chunks: HashMap<Arc<AutoMemoryPoolChunk>, Arc<RwLock<BlockAllocator>>>,
}


impl PoolAllocator {
    /// Creates a new ChunkAllocator to manage the given pool of device memory.
    pub fn new(pool: Arc<StdHostVisibleMemoryTypePool>) -> PoolAllocator {
        PoolAllocator {
            pool,
            chunks: HashMap::new()
        }
    }


    /// Allocates a new block. Uses a [BlockAllocator](::memory::allocator::BlockAllocator) to manage
    /// allocations for a given chunks, and allocates new chunks of device memory when needed.
    pub fn alloc(&mut self, size: usize, alignment: usize, pool: &Arc<AutoMemoryPoolInner>) -> AutoMemoryPoolBlock {
        for (chunk, mut block_allocator) in self.chunks.iter_mut() {
            let mut alloc_inner = block_allocator.write().unwrap();
            if let Some((block_ptr, offset)) = alloc_inner.alloc(size, alignment) {
                return AutoMemoryPoolBlock {
                    chunk: chunk.clone(),
                    allocator: block_allocator.clone(),
                    size,
                    offset,
                    block_id: block_ptr
                }
            }
            // no open spaces in that chunk, try next chunk
        }
        // no open spaces in any chunks, need to allocate new chunk
        let chunk_alloc = StdHostVisibleMemoryTypePool::alloc(&self.pool, AUTO_POOL_CHUNK_SIZE, alignment).unwrap();
        let mut chunk_id = 1;
        while self.contains_chunk(chunk_id) {
            chunk_id += 1;
        }
        let chunk = Arc::new(AutoMemoryPoolChunk {
            alloc: chunk_alloc,
            pool: pool.clone(),
            id: chunk_id
        });
        let mut block_allocator = BlockAllocator::new(AUTO_POOL_CHUNK_SIZE);
        let (block_ptr, offset) = block_allocator.alloc(size, alignment).unwrap();
        // panic on this unwrap means you tried to allocate CHUNK_SIZE on a fresh chunk. CHUNK_SIZE needs to be increased
        let allocator = Arc::new(RwLock::new(block_allocator));
        self.chunks.insert(chunk.clone(), allocator.clone());
        AutoMemoryPoolBlock {
            chunk: chunk.clone(),
            allocator,
            size,
            offset,
            block_id: block_ptr
        }
    }


    /// Gets whether a certain chunk id exists in this pool.
    pub fn contains_chunk(&self, chunk_id: usize) -> bool {
        for (chunk, _) in self.chunks.iter() {
            if chunk.id == chunk_id {
                return true;
            }
        }
        false
    }
}