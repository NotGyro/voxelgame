extern crate cgmath;
extern crate fine_grained;
extern crate fnv;
extern crate image;
extern crate noise;
extern crate rand;
extern crate smallvec;
extern crate winit;
#[macro_use] extern crate vulkano;
#[macro_use] extern crate vulkano_shader_derive;
#[macro_use] extern crate lazy_static;


#[macro_use] mod voxel;

mod memory;
mod buffer;
mod game;
mod geometry;
mod input;
mod mesh_simplifier;
mod pipeline;
mod player;
mod registry;
mod renderer;
mod renderpass;
mod shader;
mod util;
mod vulkano_win;
mod world;

fn main() {
    game::Game::new().run();
}
