// #![windows_subsystem = "windows"]

use client::{StartMode, YabClient};
use common::{comms::DEFAULT_TCP_PORT, world_type::GeneratorType};
use flexi_logger;
use gamework::video::generate_texture_atlas;
use log::*;
use num_cpus;
use rand::Rng;
use server::YabServer;
use std::{env, panic, path::Path};

enum RunMode {
    Client,
    Server,
    TexturePack,
}

fn main() {
    // Initialize logging
    flexi_logger::Logger::with_str("debug").start().unwrap();

    info!("  --== Starting YAB-World ==--");
    let args: Vec<String> = env::args().collect();

    let mut rng = rand::thread_rng();
    let num_cpus = num_cpus::get();
    info!("Number of cores detected: {}", num_cpus);
    assert!(num_cpus > 1);

    // Parse command-line options
    let mut run_mode = RunMode::Client;
    let mut client_start_mode = StartMode::Normal;
    let mut world_type = GeneratorType::Default;
    let mut seed = rng.gen::<u32>();
    for arg in args {
        let mut split_arg = arg.split("=");
        let arg_key = if let Some(arg_key) = split_arg.next() {
            arg_key
        } else {
            panic!("Cannot parse command-line arguments: empty argument");
        };
        let arg_value_opt = split_arg.next();
        match arg_key {
            "seed" => {
                if let Some(arg_value) = arg_value_opt {
                    seed = arg_value.parse::<u32>().unwrap();
                } else {
                    panic!("seed argument needs a numerical value");
                }
            }
            "type" => {
                if let Some(world_type_str) = arg_value_opt {
                    world_type = match world_type_str {
                        "flat" => GeneratorType::Flat,
                        "water" => GeneratorType::Water,
                        "alien" => GeneratorType::Alien,
                        "default" => GeneratorType::Default,
                        _ => {
                            panic!("type argument needs a value: flat, water or default")
                        }
                    }
                } else {
                    panic!("type argument needs a value: flat, water or default");
                }
            }
            "server" => run_mode = RunMode::Server,
            "new" => client_start_mode = StartMode::QuickNewWorld,
            "continue" => client_start_mode = StartMode::Continue,
            "pack" => run_mode = RunMode::TexturePack,
            _ => {}
        }
    }

    match run_mode {
        RunMode::Client => {
            let mut client = YabClient::new(client_start_mode, world_type);
            if let Err(e) = client.run() {
                error!("{}", common::error::failure_to_string(e));
            }
        }
        RunMode::Server => {
            // In case that no remote connection is desired use 127.0.0.1 instead of 0.0.0.0
            let server_address = format!("0.0.0.0:{}", DEFAULT_TCP_PORT);
            let mut server = YabServer::new(&server_address);
            server.run(true, seed, "Command-line server".to_string(), world_type);
        }
        RunMode::TexturePack => {
            info!("Packing texture atlas");
            generate_texture_atlas(
                &Path::new("client/assets/block_textures"),
                &Path::new("client/assets/atlas/blocks.png"),
                &Path::new("client/assets/atlas/blocks.json"),
            );
        }
    }
    info!("Exiting main");
}
