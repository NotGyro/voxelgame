#![allow(dead_code)]

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
#[macro_use] extern crate log;
extern crate string_cache;
extern crate linear_map;
extern crate crossbeam;
extern crate serde;

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

extern crate clap;
use clap::{Arg, App};

fn main() {
    let matches = App::new("Gestalt Engine").arg(Arg::with_name("server")
                               .short("s")
                               .long("server")
                               .help("Starts a server version of this engine. No graphics."))
                               .get_matches();
    let server_mode : bool = matches.is_present("server");
    match util::logger::init_logger() {
        Ok(_) => {},
        Err(error) => { println!("Unable to initialize logger. Reason: {}. Closing application.", error); return; }
    }
    game::Game::new(server_mode).run();
}