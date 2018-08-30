use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::BuildHasherDefault;
use std::sync::{Arc, RwLock};
use std::sync::Mutex;

use vulkano::device::Device;
use vulkano::device::DeviceOwned;
use vulkano::instance::MemoryType;
use vulkano::memory::DeviceMemory;
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::memory::MappedDeviceMemory;
use vulkano::memory::pool::AllocLayout;
use vulkano::memory::pool::MappingRequirement;
use vulkano::memory::pool::MemoryPool;
use vulkano::memory::pool::MemoryPoolAlloc;
use vulkano::memory::pool::StdHostVisibleMemoryTypePool;
use vulkano::memory::pool::StdHostVisibleMemoryTypePoolAlloc;
use fnv::FnvHasher;

use allocator::{BlockAllocator, BlockId};


/// Chunk size in bytes
const CHUNK_SIZE: usize = 1024 * 1024 * 64;


#[derive(Debug)]
pub struct PoolAllocator {
    pub pool: Arc<StdHostVisibleMemoryTypePool>,
    pub chunks: HashMap<Arc<AutoMemoryPoolChunk>, Arc<RwLock<BlockAllocator>>>,
}


impl PoolAllocator {
    pub fn new(pool: Arc<StdHostVisibleMemoryTypePool>) -> PoolAllocator {
        PoolAllocator {
            pool,
            chunks: HashMap::new()
        }
    }


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
        let chunk_alloc = StdHostVisibleMemoryTypePool::alloc(&self.pool, CHUNK_SIZE, alignment).unwrap();
        let mut chunk_id = 1;
        while self.contains_chunk(chunk_id) {
            chunk_id += 1;
        }
        let chunk = Arc::new(AutoMemoryPoolChunk {
            alloc: chunk_alloc,
            pool: pool.clone(),
            id: chunk_id
        });
        let mut block_allocator = BlockAllocator::new(CHUNK_SIZE);
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

    pub fn contains_chunk(&self, chunk_id: usize) -> bool {
        for (chunk, _) in self.chunks.iter() {
            if chunk.id == chunk_id {
                return true;
            }
        }
        false
    }
}


#[derive(Debug)]
pub struct AutoMemoryPoolInner {
    device: Arc<Device>,

    // For each memory type index, stores the associated pool.
    pools:
    Arc<Mutex<HashMap<(u32, AllocLayout, MappingRequirement), PoolAllocator, BuildHasherDefault<FnvHasher>>>>,
}

// HACK: using newtype to work around implementing foreign trait on Arc<_>
#[derive(Debug)]
pub struct AutoMemoryPool(pub Arc<AutoMemoryPoolInner>);

impl Clone for AutoMemoryPool {
    fn clone(&self) -> Self {
        AutoMemoryPool(self.0.clone())
    }
}

impl AutoMemoryPool {
    /// Creates a new pool.
    #[inline]
    pub fn new(device: Arc<Device>) -> AutoMemoryPool {
        let cap = device.physical_device().memory_types().len();
        let hasher = BuildHasherDefault::<FnvHasher>::default();

        AutoMemoryPool(Arc::new(AutoMemoryPoolInner {
            device: device.clone(),
            pools: Arc::new(Mutex::new(HashMap::with_capacity_and_hasher(cap, hasher))),
        }))
    }
}

unsafe impl MemoryPool for AutoMemoryPool {
    type Alloc = AutoMemoryPoolBlock;

    fn alloc_generic(&self, memory_type: MemoryType, size: usize, alignment: usize,
                     layout: AllocLayout, map: MappingRequirement)
                     -> Result<AutoMemoryPoolBlock, DeviceMemoryAllocError> {
        let mut pools = self.0.pools.lock().unwrap();

        if !memory_type.is_host_visible() {
            panic!("AutoMemoryPool only works with host-visible memory!");
        }

        match pools.entry((memory_type.id(), layout, map)) {
            // existing pool and allocator
            Entry::Occupied(mut entry) => {
                let mut pool_allocator = entry.get_mut();
                Ok(pool_allocator.alloc(size, alignment, &self.0))
            },
            // create new pool and allocator
            Entry::Vacant(entry) => {
                let pool = StdHostVisibleMemoryTypePool::new(self.0.device.clone(), memory_type);
                let mut pool_allocator = PoolAllocator::new(pool.clone());
                let block = pool_allocator.alloc(size, alignment, &self.0);
                entry.insert(pool_allocator);
                Ok(block)
            },
        }
    }
}

unsafe impl DeviceOwned for AutoMemoryPool {
    #[inline]
    fn device(&self) -> &Arc<Device> {
        &self.0.device
    }
}


#[derive(Debug)]
pub struct AutoMemoryPoolChunk {
    alloc: StdHostVisibleMemoryTypePoolAlloc,
    pool: Arc<AutoMemoryPoolInner>,
    id: usize
}
impl PartialEq for AutoMemoryPoolChunk {
    fn eq(&self, other: &AutoMemoryPoolChunk) -> bool {
        self.id == other.id
    }
}
impl Eq for AutoMemoryPoolChunk {}
impl ::std::hash::Hash for AutoMemoryPoolChunk {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.id);
    }
}


#[derive(Debug)]
pub struct AutoMemoryPoolBlock {
    chunk: Arc<AutoMemoryPoolChunk>,
    allocator: Arc<RwLock<BlockAllocator>>,
    size: usize,
    offset: usize,
    block_id: BlockId
}
#[allow(dead_code)]
impl AutoMemoryPoolBlock {
    #[inline]
    pub fn size(&self) -> usize { self.size }
}
unsafe impl MemoryPoolAlloc for AutoMemoryPoolBlock {
    #[inline]
    fn mapped_memory(&self) -> Option<&MappedDeviceMemory> { Some(self.chunk.alloc.memory()) }
    #[inline]
    fn memory(&self) -> &DeviceMemory { self.chunk.alloc.memory().as_ref() }
    #[inline]
    fn offset(&self) -> usize { self.chunk.alloc.offset() + self.offset }
}
impl Drop for AutoMemoryPoolBlock {
    fn drop(&mut self) {
        let mut a = self.allocator.write().unwrap();
        a.free(&self.block_id);
    }
}
