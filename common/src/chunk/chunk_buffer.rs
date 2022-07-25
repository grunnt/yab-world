use crate::block::*;
use crate::chunk::*;
use std::collections::HashMap;

pub struct ChunkBuffer {
    pub columns: HashMap<ChunkColumnPos, ChunkColumn>,
}

impl ChunkBuffer {
    pub fn new() -> ChunkBuffer {
        ChunkBuffer {
            columns: HashMap::new(),
        }
    }

    // Get a block in a chunk directly
    pub fn get_block(&self, wbx: i16, wby: i16, wbz: i16) -> Block {
        let cp = ChunkPos::from_world_block_coords(wbx, wby, wbz);
        if let Some(chunk) = self.get_chunk_pos(cp) {
            if chunk.is_initialized() {
                return chunk.get_block_world(wbx, wby, wbz);
            } else {
                return AIR_BLOCK;
            }
        }
        AIR_BLOCK
    }

    // Set a block in a chunk directly, used for collision detection when building
    pub fn set_block(&mut self, wbx: i16, wby: i16, wbz: i16, block: Block) {
        let cp = ChunkPos::from_world_block_coords(wbx, wby, wbz);
        if let Some(chunk) = self.get_mut_chunk_pos(cp) {
            chunk.set_block_world(wbx, wby, wbz, block);
        }
    }

    pub fn get_chunk(&self, wcx: i16, wcy: i16, wcz: i16) -> Option<&Chunk> {
        if let Some(column) = self.columns.get(&ChunkColumnPos::new(wcx, wcy)) {
            column.chunks.get(wcz as usize)
        } else {
            None
        }
    }

    pub fn get_chunk_pos(&self, chunk_pos: ChunkPos) -> Option<&Chunk> {
        self.get_chunk(chunk_pos.x, chunk_pos.y, chunk_pos.z)
    }

    pub fn get_mut_chunk(&mut self, wcx: i16, wcy: i16, wcz: i16) -> Option<&mut Chunk> {
        if let Some(column) = self.columns.get_mut(&ChunkColumnPos::new(wcx, wcy)) {
            column.chunks.get_mut(wcz as usize)
        } else {
            None
        }
    }

    pub fn get_mut_chunk_pos(&mut self, chunk_pos: ChunkPos) -> Option<&mut Chunk> {
        self.get_mut_chunk(chunk_pos.x, chunk_pos.y, chunk_pos.z)
    }

    /// Check if a point is inside a loaded chunk
    pub fn is_pos_in_loaded_chunk(&self, wx: f32, wy: f32) -> bool {
        let col = ChunkColumnPos::from_world_pos(wx, wy);
        if let Some(col) = self.get_column(col.x, col.y) {
            if col.is_stored() {
                return true;
            }
        }
        false
    }

    pub fn is_column_loaded(&self, col: &ChunkColumnPos) -> bool {
        self.columns.contains_key(col)
    }

    pub fn is_column_requested(&self, col: &ChunkColumnPos) -> bool {
        if let Some(column) = self.columns.get(col) {
            column.status() != ColumnStatus::New
        } else {
            false
        }
    }

    pub fn get_column(&self, wcx: i16, wcy: i16) -> Option<&ChunkColumn> {
        self.columns.get(&ChunkColumnPos::new(wcx, wcy))
    }

    pub fn get_column_pos(&self, col: &ChunkColumnPos) -> Option<&ChunkColumn> {
        self.columns.get(col)
    }

    pub fn get_column_clone(&self, wcx: i16, wcy: i16) -> Option<ChunkColumn> {
        if let Some(column) = self.columns.get(&ChunkColumnPos::new(wcx, wcy)) {
            Some(column.clone())
        } else {
            None
        }
    }

    pub fn get_mut_column(&mut self, wcx: i16, wcy: i16) -> Option<&mut ChunkColumn> {
        self.columns.get_mut(&ChunkColumnPos::new(wcx, wcy))
    }

    pub fn store_column(&mut self, column: ChunkColumn) {
        self.columns.insert(column.col, column);
    }

    pub fn are_all_neighbours_stored(&self, col: ChunkColumnPos) -> bool {
        for dx in -1..2 {
            for dy in -1..2 {
                if let Some(column) = self.get_column(col.x + dx, col.y + dy) {
                    if !column.is_stored() {
                        // Not ready yet
                        return false;
                    }
                    assert!(!column.chunks.is_empty());
                } else {
                    // Not found
                    return false;
                }
            }
        }
        true
    }

    pub fn are_all_neighbours_propagated(&self, col: ChunkColumnPos) -> bool {
        for dx in -1..2 {
            for dy in -1..2 {
                if let Some(column) = self.get_column(col.x + dx, col.y + dy) {
                    if !column.is_propagated() {
                        // Not ready yet
                        return false;
                    }
                    assert!(!column.chunks.is_empty());
                } else {
                    // Not found
                    return false;
                }
            }
        }
        true
    }

    // Search from top of map to bottom at specified x,y to find first non-empty block z coordinate
    pub fn get_top_z(&self, x: i16, y: i16) -> i16 {
        let mut cp = ChunkPos::from_world_block_coords(x, y, 0);
        let lbx = x.rem_euclid(CHUNK_SIZE as i16);
        let lby = y.rem_euclid(CHUNK_SIZE as i16);
        for cz in (0..WORLD_HEIGHT_CHUNKS as i16).rev() {
            cp.z = cz;
            if let Some(chunk) = self.get_chunk_pos(cp) {
                if chunk.is_initialized() {
                    for lbz in (0..CHUNK_SIZE).rev() {
                        if chunk.get_block(lbx as usize, lby as usize, lbz).kind() != AIR_BLOCK_KIND
                        {
                            return cz * CHUNK_SIZE as i16 + lbz as i16;
                        }
                    }
                }
            }
        }
        WORLD_HEIGHT_BLOCKS
    }
}
