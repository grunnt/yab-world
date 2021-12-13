mod chunk;
pub mod chunk_buffer;
mod chunkcolumn;
mod chunkpos;

pub const WORLD_HEIGHT_CHUNKS: usize = 64;
pub const WORLD_HEIGHT_BLOCKS: i16 = (WORLD_HEIGHT_CHUNKS * CHUNK_SIZE) as i16;
pub const CHUNK_SIZE: usize = 16;
pub const PADDED_CHUNK_SIZE: usize = CHUNK_SIZE + 2;
pub const BIT_SHIFT_Y: usize = 4;
pub const BIT_SHIFT_X: usize = 8;
// How many columns should a player move before the column buffer scrolls?
pub const COLUMN_SCROLL_STEP: i16 = 2;
pub const REGION_SIZE_BLOCKS: i16 = 512;

pub use chunk::Chunk;
pub use chunkcolumn::ChunkColumn;
pub use chunkcolumn::ChunkColumnPos;
pub use chunkcolumn::ColumnStatus;
pub use chunkpos::{ChunkPos, ChunkRegionPos};
