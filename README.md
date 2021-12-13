# YAB-World

Yet Another Block-World. I made this to learn more about Rust, OpenGL and game development.

[Read more about this on my blog](https://www.basvs.dev/blog/yab-world).

## How to run from source

Simply check out the code and run using:
```
cargo run
```

The following optional arguments can be passed:
- `server`: start a headless server
  - `seed`: set the seed to use for the server
  - `type`: set the world type to use for the server (`flat`, `water`, `default`).
- `new`: start the client & server in a new world (handy for quick iteration in development)
- `continue`: start the client & server and continue the previous world (handy for quick iteration in development)

If no command-line arguments are passed the client starts in the main menu. For joining a local server use IP address `127.1.1.1` instead of `localhost`.

## Packing block textures

The texture files used in rendering the blocks are stored in `client\assets\block_textures`. If any changes are made, these should be "packed" into a single file for performance reasons. A texture-packer is built into the application.

This can be done by running yab-world with the `pack` argument. E.g. 
```
cargo run pack
```

## The code 

The main entrypoint for the application is `src/main.rs`. From here the server and / or client are launched as needed.

Code organization:
```
/client: rendering & user interaction
/server: back-end that generates and manages the world and player connections
/common: yab-world specific code shared by client and server
/gamework: game "engine" powering the client
```