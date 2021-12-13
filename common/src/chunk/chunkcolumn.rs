use crate::chunk::*;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct ChunkColumnPos {
    pub x: i16,
    pub y: i16,
}

impl ChunkColumnPos {
    pub fn new(x: i16, y: i16) -> ChunkColumnPos {
        ChunkColumnPos { x, y }
    }

    pub fn from_chunk_pos(cp: ChunkPos) -> ChunkColumnPos {
        ChunkColumnPos { x: cp.x, y: cp.y }
    }

    pub fn dist_squared_from(&self, other: &ChunkColumnPos) -> i16 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        dx * dx + dy * dy
    }

    pub fn from_world_block_coords(wbx: i16, wby: i16) -> ChunkColumnPos {
        ChunkColumnPos {
            x: wbx >> 4,
            y: wby >> 4,
        }
    }

    pub fn from_world_pos(wx: f32, wy: f32) -> ChunkColumnPos {
        ChunkColumnPos {
            x: (wx / CHUNK_SIZE as f32).floor() as i16,
            y: (wy / CHUNK_SIZE as f32).floor() as i16,
        }
    }

    pub fn xp(&self) -> ChunkColumnPos {
        ChunkColumnPos {
            x: self.x + 1,
            y: self.y,
        }
    }

    pub fn xm(&self) -> ChunkColumnPos {
        ChunkColumnPos {
            x: self.x - 1,
            y: self.y,
        }
    }

    pub fn yp(&self) -> ChunkColumnPos {
        ChunkColumnPos {
            x: self.x,
            y: self.y + 1,
        }
    }

    pub fn ym(&self) -> ChunkColumnPos {
        ChunkColumnPos {
            x: self.x,
            y: self.y - 1,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ColumnStatus {
    New,        // Uninitialized new column
    Requested,  // Requested for generation
    Received,   // Received the blocks
    Stored,     // Stored the blocks
    Propagated, // Propagated the light
    Meshed,     // Meshed the column
}

#[derive(Debug, Clone)]
pub struct ChunkColumn {
    pub col: ChunkColumnPos,
    status: ColumnStatus,
    pub chunks: Vec<Chunk>,
}

impl ChunkColumn {
    pub fn new(col: ChunkColumnPos, status: ColumnStatus, chunks: Vec<Chunk>) -> ChunkColumn {
        ChunkColumn {
            col,
            status,
            chunks,
        }
    }

    pub fn status(&self) -> ColumnStatus {
        self.status
    }

    pub fn is_stored(&self) -> bool {
        self.status == ColumnStatus::Stored
            || self.status() == ColumnStatus::Propagated
            || self.status() == ColumnStatus::Meshed
    }

    pub fn is_propagated(&self) -> bool {
        self.status() == ColumnStatus::Propagated || self.status() == ColumnStatus::Meshed
    }

    pub fn set_status(&mut self, status: ColumnStatus) {
        self.status = status;
    }
}
