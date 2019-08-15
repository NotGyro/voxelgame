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
use std::net::{IpAddr, SocketAddr};

fn main() {
    let matches = App::new("Gestalt Engine").arg(Arg::with_name("server")
                                .short("s")
                                .long("server")
                                .help("Starts a server version of this engine. No graphics."))
                                .arg(Arg::with_name("join")
                                .short("j")
                                .long("join")
                                .value_name("IP")
                                .help("Joins a server at the selected IP address.")
                                .takes_value(true))
                                .get_matches();

    let server_mode : bool = matches.is_present("server");

    let join_ip = matches.value_of("INPUT");
    if join_ip.is_some() && server_mode {
        println!("Cannot host a server that also joins a server.");
        return;
    }
    let mut mode = game::GameMode::Singleplayer;
    if server_mode {
        mode = game::GameMode::Server("127.0.0.1:17242".parse().unwrap());
    } else if join_ip.is_some() { 
        mode = game::GameMode::JoinServer(SocketAddr::new(join_ip.unwrap().parse().unwrap(), 17242));
    }

    match util::logger::init_logger() {
        Ok(_) => {},
        Err(error) => { println!("Unable to initialize logger. Reason: {}. Closing application.", error); return; }
    }
    game::Game::new(mode).run();
}