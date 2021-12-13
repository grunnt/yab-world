use crate::render::*;
use common::block::*;
use common::chunk::chunk_buffer::ChunkBuffer;
use common::chunk::*;

pub struct WorldMesher {
    block_mesher: BlockMesher,
}

impl WorldMesher {
    pub fn new(block_registry: BlockRegistry) -> WorldMesher {
        WorldMesher {
            block_mesher: BlockMesher::new(block_registry),
        }
    }

    pub fn mesh_column(
        &self,
        col: &ChunkColumnPos,
        buffer: &ChunkBuffer,
    ) -> Vec<(Vec<BlockVertex>, Vec<BlockVertex>)> {
        let mut col_vertices: Vec<(Vec<BlockVertex>, Vec<BlockVertex>)> = Vec::new();
        for z in 0..WORLD_HEIGHT_CHUNKS {
            col_vertices.push(self.mesh_chunk(ChunkPos::new(col.x, col.y, z as i16), buffer));
        }
        col_vertices
    }

    pub fn mesh_chunk(
        &self,
        cp: ChunkPos,
        buffer: &ChunkBuffer,
    ) -> (Vec<BlockVertex>, Vec<BlockVertex>) {
        self.block_mesher.mesh_chunk(cp, buffer)
    }
}
