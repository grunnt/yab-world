# YAB-World

Yet Another Block-World. I made this to learn more about Rust, OpenGL and game development.

[Read more about this on my website](https://www.basvs.dev/projects/yab-world).

You can download and run YAB-World for Windows [here](https://github.com/grunnt/yab-world/releases).

Block textures are based on the [Woodpecker texture pack](https://www.planetminecraft.com/texture-pack/woodpecker/) by zob.

## How to run from source

You will need to have [Rust](https://www.rust-lang.org/) installed.

Simply check out the code and run using:
```
cargo run
```

The following optional arguments can be passed:
- `server`: start a headless server
  - `seed`: set the seed to use for the server
  - `type`: set the world type to use for the server (`flat`, `water`, `alien`, `default`).
- `new`: start the client & server in a new world (handy for quick iteration in development)
- `continue`: start the client & server and continue the previous world (handy for quick iteration in development)

If no command-line arguments are passed the client starts in the main menu.

## Updating blocks and textures

The texture files used in rendering the blocks are stored in `client\assets\block_textures`. If any changes are made, these should be "packed" into a single file for performance reasons. A texture-packer is built into the application.

This can be done by running yab-world with the `pack` argument. E.g. 
```
cargo run pack
```

In addition if block textures are changed, there is a new block or a block gets a different texture the "preview" pictures need to be re-generated using the following command:

```
cargo run block_previews
```

These commands also work on the built executable.

## The code 

I wrote this in the [Rust language](https://www.rust-lang.org) version 1.55. No game engine was used, just [glutin](https://docs.rs/glutin/latest/glutin) for windowing and OpenGL for rendering. OpenGL bindings are generated using the [gl_generator](https://docs.rs/gl_generator/latest/gl_generator) crate.

YAB-World suffers from Not Invented Here syndrome intentionally: this code was written as a learning experience. Still, there may be some things here that may help others.

The main entrypoint for the application is `src/main.rs`. From here the server and / or client are launched as needed.

Code organization:
```
/client: rendering & user interaction
/server: back-end that generates and manages the world and player connections
/common: yab-world specific code shared by client and server
/gamework: game "engine" powering the client
```