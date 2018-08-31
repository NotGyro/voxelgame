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

mod allocator;
mod buffer;
mod game;
mod geometry;
mod input;
mod mesh_simplifier;
mod player;
mod pool;
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
