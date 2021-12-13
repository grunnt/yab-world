use common::block::*;
use common::chunk::chunk_buffer::ChunkBuffer;
use common::chunk::*;
use std::collections::{HashSet, VecDeque};

pub const MAX_LIGHT_LEVEL: u8 = 15;

pub trait LightHandler {
    fn propagate_chunk_light(
        &mut self,
        cp: ChunkPos,
        dirty_chunks: &mut HashSet<ChunkPos>,
        block_registry: &BlockRegistry,
    );

    fn propagate_light(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        dirty_chunks: &mut HashSet<ChunkPos>,
    );

    fn propagate_light_queue(
        &mut self,
        start_queue: Vec<(i16, i16, i16)>,
        dirty_chunks: &mut HashSet<ChunkPos>,
    );

    // Remove light previously propagated from or through this block.
    fn remove_block_and_light(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        light_level: u8,
        new_block: Block,
        dirty_chunks: &mut HashSet<ChunkPos>,
    );

    fn light_removal_helper(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        light_level: u8,
        dirty_chunks: &mut HashSet<ChunkPos>,
        queue: &mut VecDeque<(i16, i16, i16, u8)>,
        propagate_queue: &mut Vec<(i16, i16, i16)>,
    );

    fn propagate_light_after_removal(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        dirty_chunks: &mut HashSet<ChunkPos>,
    );
}

impl LightHandler for ChunkBuffer {
    // ---------------------------------------------------------------------------------------------------------------
    // -- Light blocks
    // ---------------------------------------------------------------------------------------------------------------

    /// Propagate the light of all the light blocks in a chunk
    /// Use only on newly generated chunks
    fn propagate_chunk_light(
        &mut self,
        cp: ChunkPos,
        dirty_chunks: &mut HashSet<ChunkPos>,
        block_registry: &BlockRegistry,
    ) {
        let chunk = self.get_mut_chunk_pos(cp).unwrap();
        if chunk.is_normal() {
            let mut lights = Vec::new();
            for lbx in 0..CHUNK_SIZE {
                for lby in 0..CHUNK_SIZE {
                    for lbz in 0..CHUNK_SIZE {
                        let wbx = cp.x * CHUNK_SIZE as i16 + lbx as i16;
                        let wby = cp.y * CHUNK_SIZE as i16 + lby as i16;
                        let wbz = cp.z * CHUNK_SIZE as i16 + lbz as i16;
                        let mut block = chunk.get_block(lbx, lby, lbz);
                        let block_def = block_registry.get(block.kind());
                        if block_def.light > 0 {
                            block.set_light(block_def.light);
                            chunk.set_block(lbx, lby, lbz, block);
                            lights.push((wbx, wby, wbz));
                        }
                    }
                }
            }
            for (wbx, wby, wbz) in lights {
                self.propagate_light(wbx, wby, wbz, dirty_chunks);
            }
        }
    }

    fn propagate_light(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        dirty_chunks: &mut HashSet<ChunkPos>,
    ) {
        let mut queue = Vec::new();
        queue.push((wbx, wby, wbz));
        self.propagate_light_queue(queue, dirty_chunks);
    }

    /// Propagate light from a block outwards
    // Based on https://www.seedofandromeda.com/blogs/29-fast-flood-fill-lighting-in-a-blocky-voxel-game-pt-1
    fn propagate_light_queue(
        &mut self,
        start_queue: Vec<(i16, i16, i16)>,
        dirty_chunks: &mut HashSet<ChunkPos>,
    ) {
        let mut queue = Vec::new();
        for s in start_queue {
            queue.push(s);
        }
        while !queue.is_empty() {
            let (x, y, z) = queue.pop().unwrap();
            let cp = ChunkPos::from_world_block_coords(x, y, z);
            let chunk = self.get_chunk_pos(cp).unwrap();
            let light_level = chunk.get_block_world(x, y, z).get_light();
            light_helper(self, x + 1, y, z, light_level, dirty_chunks, &mut queue);
            light_helper(self, x - 1, y, z, light_level, dirty_chunks, &mut queue);
            light_helper(self, x, y + 1, z, light_level, dirty_chunks, &mut queue);
            light_helper(self, x, y - 1, z, light_level, dirty_chunks, &mut queue);
            light_helper(self, x, y, z + 1, light_level, dirty_chunks, &mut queue);
            light_helper(self, x, y, z - 1, light_level, dirty_chunks, &mut queue);
        }
    }

    // Remove light previously propagated from this block.
    fn remove_block_and_light(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        light_level: u8,
        new_block: Block,
        dirty_chunks: &mut HashSet<ChunkPos>,
    ) {
        let mut queue = VecDeque::new();
        let mut propagate_queue = Vec::new();
        let mut empty_block = Block::empty_block();
        empty_block.set_light(light_level);
        self.set_block(wbx, wby, wbz, empty_block);
        // First enqueue neighbours
        self.light_removal_helper(
            wbx,
            wby,
            wbz,
            light_level + 1,
            dirty_chunks,
            &mut queue,
            &mut propagate_queue,
        );
        // Now remove the light block
        let cp = ChunkPos::from_world_block_coords(wbx, wby, wbz);
        let chunk = self.get_mut_chunk_pos(cp).unwrap();
        chunk.set_block_world(wbx, wby, wbz, new_block);
        // Remove all light from this source
        while !queue.is_empty() {
            let (wbx, wby, wbz, light_level) = queue.pop_front().unwrap();
            self.light_removal_helper(
                wbx - 1,
                wby,
                wbz,
                light_level,
                dirty_chunks,
                &mut queue,
                &mut propagate_queue,
            );
            self.light_removal_helper(
                wbx + 1,
                wby,
                wbz,
                light_level,
                dirty_chunks,
                &mut queue,
                &mut propagate_queue,
            );
            self.light_removal_helper(
                wbx,
                wby - 1,
                wbz,
                light_level,
                dirty_chunks,
                &mut queue,
                &mut propagate_queue,
            );
            self.light_removal_helper(
                wbx,
                wby + 1,
                wbz,
                light_level,
                dirty_chunks,
                &mut queue,
                &mut propagate_queue,
            );
            self.light_removal_helper(
                wbx,
                wby,
                wbz - 1,
                light_level,
                dirty_chunks,
                &mut queue,
                &mut propagate_queue,
            );
            self.light_removal_helper(
                wbx,
                wby,
                wbz + 1,
                light_level,
                dirty_chunks,
                &mut queue,
                &mut propagate_queue,
            );
        }
        // Propagate back light from other sources
        for (wbx, wby, wbz) in propagate_queue {
            self.propagate_light(wbx, wby, wbz, dirty_chunks);
        }
    }

    fn light_removal_helper(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        light_level: u8,
        dirty_chunks: &mut HashSet<ChunkPos>,
        queue: &mut VecDeque<(i16, i16, i16, u8)>,
        propagate_queue: &mut Vec<(i16, i16, i16)>,
    ) {
        let cp = ChunkPos::from_world_block_coords(wbx, wby, wbz);
        let mut block = self.get_block(wbx, wby, wbz);
        //let block_def = self.block_registry.get(block.kind());
        if block.is_transparent() {
            // || block_def.light > 0
            let block_light = block.get_light();
            if block_light != 0 && block_light < light_level {
                block.set_light(0);
                assert!(block.get_light() == 0);
                self.set_block(wbx, wby, wbz, block);
                dirty_chunks.insert(cp);
                queue.push_back((wbx, wby, wbz, block_light));
            } else if block_light >= light_level {
                propagate_queue.push((wbx, wby, wbz));
            }
        }
    }
    fn propagate_light_after_removal(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        dirty_chunks: &mut HashSet<ChunkPos>,
    ) {
        let mut queue = Vec::new();
        if self.get_block(wbx + 1, wby, wbz).get_light() > 2 {
            queue.push((wbx + 1, wby, wbz));
        }
        if self.get_block(wbx - 1, wby, wbz).get_light() > 2 {
            queue.push((wbx - 1, wby, wbz));
        }
        if self.get_block(wbx, wby + 1, wbz).get_light() > 2 {
            queue.push((wbx, wby + 1, wbz));
        }
        if self.get_block(wbx, wby - 1, wbz).get_light() > 2 {
            queue.push((wbx, wby - 1, wbz));
        }
        if self.get_block(wbx, wby, wbz + 1).get_light() > 2 {
            queue.push((wbx, wby, wbz + 1));
        }
        if self.get_block(wbx, wby, wbz - 1).get_light() > 2 {
            queue.push((wbx, wby, wbz - 1));
        }
        self.propagate_light_queue(queue, dirty_chunks);
    }
}

// Helper for propagate_light that propagates light 1 block
fn light_helper(
    chunks: &mut ChunkBuffer,
    x: i16,
    y: i16,
    z: i16,
    light_level: u8,
    dirty_chunks: &mut HashSet<ChunkPos>,
    queue: &mut Vec<(i16, i16, i16)>,
) {
    let cp = ChunkPos::from_world_block_coords(x, y, z);
    if cp.z < 0 || cp.z >= WORLD_HEIGHT_CHUNKS as i16 {
        return;
    }
    let chunk = chunks.get_mut_chunk_pos(cp).unwrap();
    let lbx = (x - cp.x * CHUNK_SIZE as i16) as usize;
    let lby = (y - cp.y * CHUNK_SIZE as i16) as usize;
    let lbz = (z - cp.z * CHUNK_SIZE as i16) as usize;
    let mut block = chunk.get_block(lbx, lby, lbz);
    let block_light = block.get_light();
    if block.is_transparent() && block_light + 2 <= light_level {
        block.set_light(light_level - 1);
        chunk.set_block(lbx, lby, lbz, block);
        dirty_chunks.insert(cp);
        queue.push((x, y, z));
    }
}
