// TODO: A VoxelBatchHelper trait with a bunch of methods with default implementations for batch actions on voxel storages.
// These could then be overridden by implementations which would have greater knowledge of what's going on
// behind the hood, yielding optimization benefits. (Hopefully.)

// I considered adding these as methods with default implementations on VoxelStorage itself, 
// but considering that could... does Rust have trait inheritance cycle problems? TODO: Figure that out.