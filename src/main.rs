extern crate cgmath;
extern crate image;
extern crate winit;
extern crate rand;
extern crate noise;

#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;

mod game;
mod geometry;
mod input;
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
