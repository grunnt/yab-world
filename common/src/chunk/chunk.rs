use crate::block::*;
use crate::chunk::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    pub pos: ChunkPos,
    pub blocks: Vec<Block>,
}

impl Chunk {
    /// Create an uninitialized chunk
    pub fn new_uninitialized(pos: ChunkPos) -> Chunk {
        Chunk {
            pos,
            blocks: Vec::new(),
        }
    }

    /// Create a solid chunk (i.e. has 1 block type)
    pub fn new_solid(pos: ChunkPos, block: Block) -> Chunk {
        let mut blocks = Vec::with_capacity(1);
        blocks.push(block);
        Chunk { pos, blocks }
    }

    /// Create a normal chunk
    pub fn new_normal(pos: ChunkPos, block: Block) -> Chunk {
        let blocks = vec![block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        Chunk { pos, blocks }
    }

    pub fn is_initialized(&self) -> bool {
        self.blocks.len() > 0
    }

    pub fn is_normal(&self) -> bool {
        self.blocks.len() > 1
    }

    pub fn is_solid(&self) -> bool {
        self.blocks.len() == 1
    }

    pub fn get_block(&self, x_rel: usize, y_rel: usize, z_rel: usize) -> Block {
        debug_assert!(self.is_initialized());
        if self.is_solid() {
            self.blocks[0]
        } else {
            debug_assert!(x_rel < CHUNK_SIZE);
            debug_assert!(y_rel < CHUNK_SIZE);
            debug_assert!(z_rel < CHUNK_SIZE);
            let index = z_rel | y_rel << BIT_SHIFT_Y | x_rel << BIT_SHIFT_X;
            self.blocks[index]
        }
    }

    pub fn get_block_world(&self, wbx: i16, wby: i16, wbz: i16) -> Block {
        let lbx = (wbx - self.pos.x * CHUNK_SIZE as i16) as usize;
        let lby = (wby - self.pos.y * CHUNK_SIZE as i16) as usize;
        let lbz = (wbz - self.pos.z * CHUNK_SIZE as i16) as usize;
        self.get_block(lbx, lby, lbz)
    }

    pub fn make_normal(&mut self) {
        let solid_block = self.blocks[0];
        self.blocks = vec![solid_block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    }

    pub fn set_block(&mut self, x_rel: usize, y_rel: usize, z_rel: usize, block: Block) {
        let index = z_rel | y_rel << BIT_SHIFT_Y | x_rel << BIT_SHIFT_X;
        if self.is_solid() {
            if block != self.blocks[0] {
                // Need to support different blocks in this chunk
                self.make_normal();
                self.blocks[index] = block;
            }
        } else {
            self.blocks[index] = block;
        }
    }

    pub fn set_block_world(&mut self, wbx: i16, wby: i16, wbz: i16, block: Block) {
        let lbx = (wbx - self.pos.x * CHUNK_SIZE as i16) as usize;
        let lby = (wby - self.pos.y * CHUNK_SIZE as i16) as usize;
        let lbz = (wbz - self.pos.z * CHUNK_SIZE as i16) as usize;
        self.set_block(lbx, lby, lbz, block);
    }

    pub fn set_block_unchecked(&mut self, x_rel: usize, y_rel: usize, z_rel: usize, block: Block) {
        let index = z_rel | y_rel << BIT_SHIFT_Y | x_rel << BIT_SHIFT_X;
        self.blocks[index] = block;
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
    }
}
