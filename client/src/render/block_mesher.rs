use std::collections::HashMap;

use crate::render::*;
use common::block::*;
use common::chunk::chunk_buffer::ChunkBuffer;
use common::chunk::*;

// Vertices of a cube
const VERTICES: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0], // v0
    [0.0, 1.0, 0.0], // v1
    [1.0, 1.0, 0.0], // v2
    [1.0, 0.0, 0.0], // v3
    [0.0, 0.0, 1.0], // v4
    [0.0, 1.0, 1.0], // v5
    [1.0, 1.0, 1.0], // v6
    [1.0, 0.0, 1.0], // v7
];

const FACES: [[usize; 4]; 6] = [
    [3, 7, 6, 2], // xp
    [1, 5, 4, 0], // xm
    [2, 6, 5, 1], // yp
    [0, 4, 7, 3], // ym
    [4, 5, 6, 7], // zp
    [0, 3, 2, 1], // zm
];

// Normals of the faces
// These are packed values and unpacked like this:
// vec3 unpackedNormal = vec3((Normal & 3) - 1, ((Normal >> 2) & 3) - 1, (Normal >> 4) - 1);
// 1 = 1
// 1 = 2
// 1 = 4
// 1 = 8
// 1 = 16
// 1 = 32
const NORMALS: [u8; 6] = [
    20, // xp
    22, // xm
    17, // yp
    25, // ym
    5,  // zp
    37, // zm
];

pub struct BlockMesher {
    block_texture_map: HashMap<Block, [f32; 6]>,
}

impl BlockMesher {
    pub fn new(block_registry: BlockRegistry) -> BlockMesher {
        // We need to build a mapping from block type to texture array index
        let block_texture_map = block_registry.block_texture_map().clone();
        BlockMesher { block_texture_map }
    }

    fn texture(&self, block_type: Block, face: usize) -> f32 {
        self.block_texture_map.get(&block_type).unwrap()[face]
    }

    // Generate mesh vertices
    pub fn mesh_chunk(
        &self,
        cp: ChunkPos,
        buffer: &ChunkBuffer,
    ) -> (Vec<BlockVertex>, Vec<BlockVertex>) {
        let mut vertices = Vec::new();
        let mut translucent_vertices = Vec::new();
        let offset_x = cp.x * CHUNK_SIZE as i16;
        let offset_y = cp.y * CHUNK_SIZE as i16;
        let offset_z = cp.z * CHUNK_SIZE as i16;
        // Center
        let chunk = buffer.get_chunk_pos(cp).unwrap();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block = chunk.get_block(x, y, z);
                    let light = block.get_light();
                    if block.is_transparent() {
                        // There may be solid blocks next to this one that may need faces rendered
                        if x < CHUNK_SIZE - 1 {
                            let neighbour = chunk.get_block(x + 1, y, z);
                            let vertices_opt = if neighbour.is_opaque() {
                                Some(&mut vertices)
                            } else if block.is_empty() && neighbour.is_translucent() {
                                Some(&mut translucent_vertices)
                            } else {
                                None
                            };
                            if let Some(verts) = vertices_opt {
                                self.add_face(
                                    verts,
                                    x as i16 + offset_x,
                                    y as i16 + offset_y,
                                    z as i16 + offset_z,
                                    self.texture(neighbour.kind(), FACE_XM),
                                    light,
                                    FACE_XP,
                                );
                            }
                        }
                        if x > 0 {
                            let neighbour = chunk.get_block(x - 1, y, z);
                            let vertices_opt = if neighbour.is_opaque() {
                                Some(&mut vertices)
                            } else if block.is_empty() && neighbour.is_translucent() {
                                Some(&mut translucent_vertices)
                            } else {
                                None
                            };
                            if let Some(verts) = vertices_opt {
                                self.add_face(
                                    verts,
                                    x as i16 + offset_x,
                                    y as i16 + offset_y,
                                    z as i16 + offset_z,
                                    self.texture(neighbour.kind(), FACE_XP),
                                    light,
                                    FACE_XM,
                                );
                            }
                        }
                        if y < CHUNK_SIZE - 1 {
                            let neighbour = chunk.get_block(x, y + 1, z);
                            let vertices_opt = if neighbour.is_opaque() {
                                Some(&mut vertices)
                            } else if block.is_empty() && neighbour.is_translucent() {
                                Some(&mut translucent_vertices)
                            } else {
                                None
                            };
                            if let Some(verts) = vertices_opt {
                                self.add_face(
                                    verts,
                                    x as i16 + offset_x,
                                    y as i16 + offset_y,
                                    z as i16 + offset_z,
                                    self.texture(neighbour.kind(), FACE_YM),
                                    light,
                                    FACE_YP,
                                );
                            }
                        }
                        if y > 0 {
                            let neighbour = chunk.get_block(x, y - 1, z);
                            let vertices_opt = if neighbour.is_opaque() {
                                Some(&mut vertices)
                            } else if block.is_empty() && neighbour.is_translucent() {
                                Some(&mut translucent_vertices)
                            } else {
                                None
                            };
                            if let Some(verts) = vertices_opt {
                                self.add_face(
                                    verts,
                                    x as i16 + offset_x,
                                    y as i16 + offset_y,
                                    z as i16 + offset_z,
                                    self.texture(neighbour.kind(), FACE_YP),
                                    light,
                                    FACE_YM,
                                );
                            }
                        }
                        if z < CHUNK_SIZE - 1 {
                            let neighbour = chunk.get_block(x, y, z + 1);
                            let vertices_opt = if neighbour.is_opaque() {
                                Some(&mut vertices)
                            } else if block.is_empty() && neighbour.is_translucent() {
                                Some(&mut translucent_vertices)
                            } else {
                                None
                            };
                            if let Some(verts) = vertices_opt {
                                self.add_face(
                                    verts,
                                    x as i16 + offset_x,
                                    y as i16 + offset_y,
                                    z as i16 + offset_z,
                                    self.texture(neighbour.kind(), FACE_ZM),
                                    light,
                                    FACE_ZP,
                                );
                            }
                        }
                        if z > 0 {
                            let neighbour = chunk.get_block(x, y, z - 1);
                            let vertices_opt = if neighbour.is_opaque() {
                                Some(&mut vertices)
                            } else if block.is_empty() && neighbour.is_translucent() {
                                Some(&mut translucent_vertices)
                            } else {
                                None
                            };
                            if let Some(verts) = vertices_opt {
                                self.add_face(
                                    verts,
                                    x as i16 + offset_x,
                                    y as i16 + offset_y,
                                    z as i16 + offset_z,
                                    self.texture(neighbour.kind(), FACE_ZP),
                                    light,
                                    FACE_ZM,
                                );
                            }
                        }
                    }
                }
            }
        }
        // XP side
        let xp = buffer.get_chunk_pos(cp.xp()).unwrap();
        let x = CHUNK_SIZE - 1;
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block = chunk.get_block(x, y, z);
                if block.is_transparent() {
                    // There may be solid blocks next to this one that may need faces rendered
                    let light = block.get_light();
                    let neighbour = xp.get_block(0, y, z);
                    let vertices_opt = if neighbour.is_opaque() {
                        Some(&mut vertices)
                    } else if block.is_empty() && neighbour.is_translucent() {
                        Some(&mut translucent_vertices)
                    } else {
                        None
                    };
                    if let Some(verts) = vertices_opt {
                        self.add_face(
                            verts,
                            x as i16 + offset_x,
                            y as i16 + offset_y,
                            z as i16 + offset_z,
                            self.texture(neighbour.kind(), FACE_XM),
                            light,
                            FACE_XP,
                        );
                    }
                }
            }
        }
        // XM side
        let xm = buffer.get_chunk_pos(cp.xm()).unwrap();
        let x = 0;
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block = chunk.get_block(x, y, z);
                if block.is_transparent() {
                    // There may be solid blocks next to this one that may need faces rendered
                    let light = block.get_light();
                    let neighbour = xm.get_block(CHUNK_SIZE - 1, y, z);
                    let vertices_opt = if neighbour.is_opaque() {
                        Some(&mut vertices)
                    } else if block.is_empty() && neighbour.is_translucent() {
                        Some(&mut translucent_vertices)
                    } else {
                        None
                    };
                    if let Some(verts) = vertices_opt {
                        self.add_face(
                            verts,
                            x as i16 + offset_x,
                            y as i16 + offset_y,
                            z as i16 + offset_z,
                            self.texture(neighbour.kind(), FACE_XP),
                            light,
                            FACE_XM,
                        );
                    }
                }
            }
        }
        // YP side
        let yp = buffer.get_chunk_pos(cp.yp()).unwrap();
        let y = CHUNK_SIZE - 1;
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block = chunk.get_block(x, y, z);
                if block.is_transparent() {
                    // There may be solid blocks next to this one that may need faces rendered
                    let light = block.get_light();
                    let neighbour = yp.get_block(x, 0, z);
                    let vertices_opt = if neighbour.is_opaque() {
                        Some(&mut vertices)
                    } else if block.is_empty() && neighbour.is_translucent() {
                        Some(&mut translucent_vertices)
                    } else {
                        None
                    };
                    if let Some(verts) = vertices_opt {
                        self.add_face(
                            verts,
                            x as i16 + offset_x,
                            y as i16 + offset_y,
                            z as i16 + offset_z,
                            self.texture(neighbour.kind(), FACE_YM),
                            light,
                            FACE_YP,
                        );
                    }
                }
            }
        }
        // YM side
        let ym = buffer.get_chunk_pos(cp.ym()).unwrap();
        let y = 0;
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block = chunk.get_block(x, y, z);
                if block.is_transparent() {
                    // There may be solid blocks next to this one that may need faces rendered
                    let light = block.get_light();
                    let neighbour = ym.get_block(x, CHUNK_SIZE - 1, z);
                    let vertices_opt = if neighbour.is_opaque() {
                        Some(&mut vertices)
                    } else if block.is_empty() && neighbour.is_translucent() {
                        Some(&mut translucent_vertices)
                    } else {
                        None
                    };
                    if let Some(verts) = vertices_opt {
                        self.add_face(
                            verts,
                            x as i16 + offset_x,
                            y as i16 + offset_y,
                            z as i16 + offset_z,
                            self.texture(neighbour.kind(), FACE_YP),
                            light,
                            FACE_YM,
                        );
                    }
                }
            }
        }
        // ZP side
        let zp = buffer.get_chunk_pos(cp.zp());
        let z = CHUNK_SIZE - 1;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let block = chunk.get_block(x, y, z);
                if block.is_transparent() {
                    // There may be solid blocks next to this one that may need faces rendered
                    let light = block.get_light();
                    let neighbour = if let Some(chunk) = zp {
                        chunk.get_block(x, y, 0)
                    } else {
                        Block::empty_block()
                    };
                    let vertices_opt = if neighbour.is_opaque() {
                        Some(&mut vertices)
                    } else if block.is_empty() && neighbour.is_translucent() {
                        Some(&mut translucent_vertices)
                    } else {
                        None
                    };
                    if let Some(verts) = vertices_opt {
                        self.add_face(
                            verts,
                            x as i16 + offset_x,
                            y as i16 + offset_y,
                            z as i16 + offset_z,
                            self.texture(neighbour.kind(), FACE_ZM),
                            light,
                            FACE_ZP,
                        );
                    }
                }
            }
        }
        // ZM side
        let zm = buffer.get_chunk_pos(cp.zm());
        let z = 0;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let block = chunk.get_block(x, y, z);
                if block.is_transparent() {
                    // There may be solid blocks next to this one that may need faces rendered
                    let light = block.get_light();
                    let neighbour = if let Some(chunk) = zm {
                        chunk.get_block(x, y, CHUNK_SIZE - 1)
                    } else {
                        Block::bedrock_block()
                    };
                    let vertices_opt = if neighbour.is_opaque() {
                        Some(&mut vertices)
                    } else if block.is_empty() && neighbour.is_translucent() {
                        Some(&mut translucent_vertices)
                    } else {
                        None
                    };
                    if let Some(verts) = vertices_opt {
                        self.add_face(
                            verts,
                            x as i16 + offset_x,
                            y as i16 + offset_y,
                            z as i16 + offset_z,
                            self.texture(neighbour.kind(), FACE_ZP),
                            light,
                            FACE_ZM,
                        );
                    }
                }
            }
        }
        (vertices, translucent_vertices)
    }

    // Render a face of a cube
    fn add_face(
        &self,
        vertices: &mut Vec<BlockVertex>,
        x: i16,
        y: i16,
        z: i16,
        texture_layer: f32,
        light: u8,
        face: usize,
    ) {
        // Vertices of this face in clockwise order
        let vert0 = FACES[face][0];
        let vert1 = FACES[face][1];
        let vert2 = FACES[face][2];
        let vert3 = FACES[face][3];

        let xp = x as f32;
        let yp = y as f32;
        let zp = z as f32;
        let vertex0 = BlockVertex::new(
            xp + VERTICES[vert0][0],
            yp + VERTICES[vert0][1],
            zp + VERTICES[vert0][2],
            0.0,
            1.0,
            texture_layer,
            light,
            NORMALS[face],
        );
        let vertex1 = BlockVertex::new(
            xp + VERTICES[vert1][0],
            yp + VERTICES[vert1][1],
            zp + VERTICES[vert1][2],
            0.0,
            0.0,
            texture_layer,
            light,
            NORMALS[face],
        );
        let vertex2 = BlockVertex::new(
            xp + VERTICES[vert2][0],
            yp + VERTICES[vert2][1],
            zp + VERTICES[vert2][2],
            1.0,
            0.0,
            texture_layer,
            light,
            NORMALS[face],
        );
        let vertex3 = BlockVertex::new(
            xp + VERTICES[vert3][0],
            yp + VERTICES[vert3][1],
            zp + VERTICES[vert3][2],
            1.0,
            1.0,
            texture_layer,
            light,
            NORMALS[face],
        );
        // Triangle 1
        vertices.push(vertex0);
        vertices.push(vertex1);
        vertices.push(vertex2);
        // Triangle 2
        vertices.push(vertex0);
        vertices.push(vertex2);
        vertices.push(vertex3);
    }
}
