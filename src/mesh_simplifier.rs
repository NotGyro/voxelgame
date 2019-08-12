//! Simplified mesh generator.

use std::sync::Arc;
use std::collections::HashSet;

use cgmath::Point3;
use vulkano::device::Device;

use geometry::{Mesh, VertexPositionNormalUVColor, VertexGroup};
use util::Transform;
use memory::pool::AutoMemoryPool;

use voxel::voxelstorage::*;
use voxel::voxelmath::*;
use world::block;


type VoxelTy = block::BlockID;
type Chunk = block::Chunk;
type ChunkBounds = VoxelRange<i32>;

const AIR : VoxelTy = 0;

/// Struct used internally to represent unoptimized quads.
#[derive(Clone)]
pub struct InputQuad { x: usize, y: usize, exists: bool, done: bool, pub block_id: VoxelTy }
/// Struct returned as output from the generator; represents quads in an optimized mesh.
#[derive(Debug, Clone)]
pub struct OutputQuad { pub x: usize, pub y: usize, pub w: usize, pub h: usize, width_done: bool, pub block_id: VoxelTy }


/// Simplified mesh generator.
///
/// Generates a list of quads to render a chunk, optimized using greedy meshing, and with inner faces culled.
pub struct MeshSimplifier;
#[derive(Debug, Clone)]
pub struct ChunkMeshError; // TODO

impl MeshSimplifier {
    // The bug here is negative X faces don't get generated. SPECIFICALLY negative X-facing faces.
    /// Generates a simplified mesh from the given chunk. Returns side, layer of this side (stacked), quads.
    pub fn generate_quads(chunk: &Chunk, range: ChunkBounds) -> Vec<(VoxelAxis, usize, Vec<OutputQuad>)> {
        let mut output = Vec::new();
        // Look in each direction.
        voxel_sides_unroll!(facing, {
            // Which directions AREN'T the ones we're stacking?
            let up = match facing.into() {
                VoxelAxisUnsigned::X => VoxelAxis::PosiY,
                VoxelAxisUnsigned::Y => VoxelAxis::PosiZ,
                VoxelAxisUnsigned::Z => VoxelAxis::PosiY,
            };
            let across = match facing.into() {
                VoxelAxisUnsigned::X => VoxelAxis::PosiZ,
                VoxelAxisUnsigned::Y => VoxelAxis::PosiX,
                VoxelAxisUnsigned::Z => VoxelAxis::PosiX,
            };
            let chunk_size : VoxelPos<u8> = vpos!(range.get_size().x as u8, range.get_size().y as u8, range.get_size().z as u8);
            let bounds_local : VoxelRange<u8> = VoxelRange{ lower: vpos!(0,0,0), upper: chunk_size };
            // Iterate by "layers" of each block-side from one end of the chunk to another.
            let max_layer = chunk_size.coord_for_axis(facing.into());

            for layer_l in 0 .. max_layer {
                let layer = match facing.get_sign() {
                            VoxelAxisSign::POSI => layer_l,
                            VoxelAxisSign::NEGA => max_layer - layer_l,};
                let mut input_quads = Vec::new();
                let max_y = chunk_size.coord_for_axis(up.into());
                let max_x = chunk_size.coord_for_axis(across.into());
                for y in 0..max_y {
                    for x in 0..max_x {
                        let mut point : VoxelPos<u8> = vpos!(0,0,0);
                        point.set_coord_for_axis(facing.into(), layer);
                        point.set_coord_for_axis(up.into(), y);
                        point.set_coord_for_axis(across.into(), x);
                        let adjacent_point = point.get_neighbor_unsigned(facing);
                        let voxel_maybe = chunk.get(point);
                        let exists = match voxel_maybe {
                            None => false,
                            Some(AIR) => false,
                            //Check neighbor point
                            Some(_) => match adjacent_point {
                                Ok(adj) => {
                                    //We have found our way to the end of the chunk, and the next block is past our range. Make this side solid.
                                    if !(bounds_local.contains(adj)) {
                                        true
                                    } else {
                                        // Is the neighboring block solid?
                                        match chunk.get(adj) {
                                            None => true, //End of the underlying voxel storage - should match end of chunk.
                                            Some(AIR) => true, //Air block, nothing in our way.
                                            Some(_) => false, //Solid block in the way, do not process this quad.
                                        }
                                    }
                                },
                                Err(_) => true, //Underflow. Our neighbor would have been at a negative point, but this is unsigned.
                            },
                        };
                        input_quads.push(InputQuad { x: (x as usize), y: (y as usize), exists: exists, 
                        done: false, block_id: voxel_maybe.unwrap_or(AIR), });
                    }
                }
                // Done with this slice, now process it.
                output.push((facing, layer as usize, MeshSimplifier::process_slice(input_quads, max_x as usize, max_y as usize)));
            }
        });
        output
    }

    /// Generates one 2d slice of the mesh.
    pub fn process_slice(mut input_quads: Vec<InputQuad>, slice_width : usize, slice_height : usize) -> Vec<OutputQuad> {
        let mut output_quads = Vec::new();
        let mut current_quad: Option<OutputQuad> = None;
        let mut i : usize= 0;
        while i < (slice_width*slice_height) as usize {
            let mut q = input_quads.get_mut(i).unwrap().clone();
            if current_quad.is_none() {
                if q.exists && !q.done {
                    current_quad = Some(OutputQuad { x: q.x, y: q.y, w: 1, h: 1, width_done: false, block_id: q.block_id });
                    q.done = true;
                }
                i += 1;
                continue;
            }
            let mut current = current_quad.unwrap();
            if !current.width_done {
                // is quad on the same row?
                if q.x > current.x {
                    // moving right, check for quad
                    if q.exists && !q.done && q.block_id == current.block_id {
                        q.done = true;
                        current.w += 1;
                    }
                    else {
                        // found a gap, done with right expansion
                        current.width_done = true;
                    }
                }
                else {
                    // quad below start, meaning next row, done with right expansion
                    current.width_done = true;
                }
            }
            if current.width_done {
                let mut y = current.y + 1;
                if y < slice_height {
                    loop {
                        let x_min = current.x;
                        let x_max = current.x + current.w;
                        let mut ok = true;
                        for x in x_min..x_max {
                            if !input_quads[y*slice_width+x].exists 
                                    || input_quads[y*slice_width+x].done 
                                    || input_quads[y*slice_width+x].block_id != current.block_id {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            for x in x_min..x_max {
                                input_quads[y*slice_width+x].done = true;
                            }
                            current.h += 1;
                            y += 1;
                            if y >= slice_width { break; }
                        }
                        else { break; }
                    }
                }
                output_quads.push(current);
                current_quad = None;
                continue;
            }
            i += 1;
            // when i == 16*16, loop would end without adding quad
            if i >= slice_width*slice_height {
                output_quads.push(current.clone());
                break;
            }
            current_quad = Some(current);
        }

        output_quads
    }

    /// Generates a mesh for a chunk, using [MeshSimplifier].
    pub fn generate_mesh(chunk: &Chunk, range: ChunkBounds, device: Arc<Device>, 
                                memory_pool: AutoMemoryPool) -> Result<Mesh, ChunkMeshError> {
        let quad_lists = MeshSimplifier::generate_quads(chunk, range);

        // Get all unique block ids and seperate
        let mut unique_ids = HashSet::new();
        for (_, _, list) in quad_lists.iter() {
            for quad in list.iter() {
                unique_ids.insert(quad.block_id);
            }
        }
        unique_ids.remove(&AIR); // don't generate anything for air

        let mut mesh = Mesh::new();
        /*let mut count_p_x = 0;
        let mut count_n_x = 0;
        let mut count_p_y = 0;
        let mut count_n_y = 0;
        let mut count_p_z = 0;
        let mut count_n_z = 0;
        */
        // TODO: currently iterates over the whole quad list [# of unique ids] times. for diverse
        // chunks this will get expensive. needs optimization.
        for id in unique_ids.iter() {
            let mut vertices = Vec::new() as Vec<VertexPositionNormalUVColor>;
            let mut indices = Vec::new() as Vec<u32>;
            let mut o = 0;
            for (facing, layer, list) in quad_lists.iter() {
                /*match facing {
                    VoxelAxis::PosiX => count_p_x += list.len(),
                    VoxelAxis::NegaX => count_n_x += list.len(),
                    VoxelAxis::PosiY => count_p_y += list.len(),
                    VoxelAxis::NegaY => count_n_y += list.len(),
                    VoxelAxis::PosiZ => count_p_z += list.len(),
                    VoxelAxis::NegaZ => count_n_z += list.len(),
                }*/
                for quad in list {
                    if quad.block_id != *id { continue; }
                    match facing {
                        //Positive X face gets added, negative X face goes nowhere.
                        VoxelAxis::NegaX => {
                            vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32, quad.y as f32,          quad.x as f32,], normal: [ -1.0, 0.0, 0.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32, quad.y as f32,          (quad.x+quad.w) as f32], normal: [ -1.0, 0.0, 0.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32, (quad.y+quad.h) as f32, (quad.x+quad.w) as f32], normal: [ -1.0, 0.0, 0.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32, (quad.y+quad.h) as f32, quad.x as f32], normal: [ -1.0, 0.0, 0.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                        },
                        VoxelAxis::PosiX => {
                            vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32 + 1.0, (quad.y+quad.h) as f32, quad.x as f32 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32 + 1.0, (quad.y+quad.h) as f32, (quad.x+quad.w) as f32 ], normal: [ 1.0, 0.0, 0.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32 + 1.0, quad.y as f32,          (quad.x+quad.w) as f32], normal: [ 1.0, 0.0, 0.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ *layer as f32 + 1.0, quad.y as f32,          quad.x as f32], normal: [ 1.0, 0.0, 0.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        },
                        VoxelAxis::NegaY => {
                            vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, *layer as f32, (quad.y+quad.h) as f32 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          *layer as f32, (quad.y+quad.h) as f32 ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          *layer as f32, quad.y as f32          ], normal: [ 0.0, -1.0, 0.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, *layer as f32, quad.y as f32          ], normal: [ 0.0, -1.0, 0.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        },
                        VoxelAxis::PosiY => {
                            vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          *layer as f32 + 1.0, (quad.y+quad.h) as f32 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, *layer as f32 + 1.0, (quad.y+quad.h) as f32 ], normal: [ 0.0, 1.0, 0.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, *layer as f32 + 1.0, quad.y as f32          ], normal: [ 0.0, 1.0, 0.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          *layer as f32 + 1.0, quad.y as f32          ], normal: [ 0.0, 1.0, 0.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        },
                        VoxelAxis::NegaZ => {
                            vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          (quad.y+quad.h) as f32, *layer as f32 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, (quad.y+quad.h) as f32, *layer as f32 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, quad.y as f32,          *layer as f32 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          quad.y as f32,          *layer as f32 ], normal: [ 0.0, 0.0, -1.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        },
                        VoxelAxis::PosiZ => {
                            vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, (quad.y+quad.h) as f32, *layer as f32 + 1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ quad.w as f32, 0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          (quad.y+quad.h) as f32, *layer as f32 + 1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 0.0,           0.0 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ quad.x as f32,          quad.y as f32,          *layer as f32 + 1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ 0.0,           quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                            vertices.push(VertexPositionNormalUVColor { position: [ (quad.x+quad.w) as f32, quad.y as f32,          *layer as f32 + 1.0 ], normal: [ 0.0, 0.0, 1.0 ], uv: [ quad.w as f32, quad.h as f32 ], color: [ 1.0, 1.0, 1.0 ] });
                        },
                    }
                    indices.push(0+o); indices.push(1+o); indices.push(2+o);
                    indices.push(2+o); indices.push(3+o); indices.push(0+o);
                    o += 4;
                }
            }
            mesh.vertex_groups.push(Arc::new(VertexGroup::new(vertices, indices, (*id as VoxelTy) as u8, device.clone(), memory_pool.clone())));
        }

        //println!("+x: {}, -x: {}, +y: {}, -y: {}, +z: {}, -z: {}", count_p_x, count_n_x, count_p_y, count_n_y, count_p_z, count_n_z);
        //Range.lower is currently our origin in worldspace (1 block = 1 unit), so we can just use it directly as the transform for this mesh.
        mesh.transform = Transform::from_position(Point3::new(range.lower.x as f32, range.lower.y as f32, range.lower.z as f32));

        return Ok(mesh);
    }
}