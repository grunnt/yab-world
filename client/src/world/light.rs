use common::block::*;
use common::chunk::chunk_buffer::ChunkBuffer;
use common::chunk::*;
use std::collections::{HashSet, VecDeque};

pub const MAX_LIGHT_LEVEL: u8 = 15;
pub trait LightHandler {
    fn propagate_column_sunlight(&mut self, col: ChunkColumnPos);

    fn propagate_chunk_light_and_sunlight(
        &mut self,
        cp: ChunkPos,
        dirty_chunks: &mut HashSet<ChunkPos>,
        block_registry: &BlockRegistry,
    );

    fn propagate_sunlight_if_needed(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        dirty_chunks: &mut HashSet<ChunkPos>,
    );

    fn propagate_sunlight_queue(
        &mut self,
        start_queue: Vec<(i16, i16, i16)>,
        dirty_chunks: &mut HashSet<ChunkPos>,
    );

    // Remove sunlight from this block and also remove sunlight propagated from here
    fn remove_sunlight_if_needed(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        old_block_sunlight: u8,
        dirty_chunks: &mut HashSet<ChunkPos>,
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
    // -- Light blocks and sunlight
    // ---------------------------------------------------------------------------------------------------------------

    // Propagate sunlight from the top of the map downwards for a new column of chunks
    fn propagate_column_sunlight(&mut self, col: ChunkColumnPos) {
        // Sunlight propagates from the top of the map downwards
        let mut sunlight_map = [[MAX_LIGHT_LEVEL as f32; CHUNK_SIZE]; CHUNK_SIZE];
        for lcz in (0..WORLD_HEIGHT_CHUNKS).rev() {
            let chunk = self.get_mut_chunk(col.x, col.y, lcz as i16).unwrap();
            assert!(chunk.is_initialized());
            if chunk.is_solid() {
                let mut block = chunk.get_block(0, 0, 0);
                if block.is_opaque() {
                    // Completely solid chunk, we are done propagating
                    return;
                } else {
                    // Entire chunk is lit
                    // TODO if water change column to normal and fix sunlight propagation
                    block.set_sunlight(MAX_LIGHT_LEVEL);
                    chunk.set_block_unchecked(0, 0, 0, block);
                }
            } else {
                // Go through chunk from top to bottom
                for z in (0..CHUNK_SIZE).rev() {
                    let mut has_light = false;
                    for x in 0..CHUNK_SIZE {
                        for y in 0..CHUNK_SIZE {
                            let mut block = chunk.get_block(x, y, z);
                            let light_above = sunlight_map[x][y];
                            if light_above >= 1.0 && block.is_transparent() {
                                block.set_sunlight(light_above.floor() as u8);
                                chunk.set_block_unchecked(x, y, z, block);
                                has_light = true;
                                if block.kind() == Block::water_block() {
                                    sunlight_map[x][y] = sunlight_map[x][y] * 0.5;
                                }
                            } else {
                                sunlight_map[x][y] = 0.0;
                            }
                        }
                    }
                    if !has_light {
                        // No sunlight passes through here, so no point in propagating
                        return;
                    }
                }
            }
        }
    }

    /// Propagate the light of all the light blocks in a chunk
    /// Use only on newly generated chunks
    fn propagate_chunk_light_and_sunlight(
        &mut self,
        cp: ChunkPos,
        dirty_chunks: &mut HashSet<ChunkPos>,
        block_registry: &BlockRegistry,
    ) {
        let chunk = self.get_mut_chunk_pos(cp).unwrap();
        if chunk.is_normal() {
            let mut lights = Vec::new();
            let mut sunlight_queue = Vec::new();
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
                        } else if block.get_sunlight() == MAX_LIGHT_LEVEL {
                            sunlight_queue.push((wbx, wby, wbz));
                        }
                    }
                }
            }
            for (wbx, wby, wbz) in lights {
                self.propagate_light(wbx, wby, wbz, dirty_chunks);
            }
            self.propagate_sunlight_queue(sunlight_queue, dirty_chunks);
        }
    }

    /// Propagate sunlight for the indicated block downwards
    fn propagate_sunlight_if_needed(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        dirty_chunks: &mut HashSet<ChunkPos>,
    ) {
        let this_block = self.get_block(wbx, wby, wbz);
        if this_block.is_opaque() {
            // No sunlight propagation through opaque blocks
            return;
        }
        let sunlight_top = self.get_block(wbx, wby, wbz + 1).get_sunlight();
        let mut propagate_queue = Vec::new();

        if sunlight_top == MAX_LIGHT_LEVEL {
            // 1. This block is removed and sunlight can beam down from above.
            for z in (0..wbz + 1).rev() {
                let mut block = self.get_block(wbx, wby, z);
                if block.is_opaque() {
                    break;
                } else {
                    block.set_sunlight(MAX_LIGHT_LEVEL);
                    self.set_block(wbx, wby, z, block);
                    dirty_chunks.insert(ChunkPos::from_world_block_coords(wbx, wby, z));
                    propagate_queue.push((wbx, wby, z));
                }
            }
        } else {
            // 2. This block is removed, and neighbours may have sunlight to propagate into here.
            propagate_queue.push((wbx + 1, wby, wbz));
            propagate_queue.push((wbx - 1, wby, wbz));
            propagate_queue.push((wbx, wby + 1, wbz));
            propagate_queue.push((wbx, wby - 1, wbz));
            propagate_queue.push((wbx, wby, wbz + 1));
            propagate_queue.push((wbx, wby, wbz - 1));
        }

        // Now propagate sunlight from the downward beam to the sides
        self.propagate_sunlight_queue(propagate_queue, dirty_chunks);
    }

    /// Propagate sunlight from sunlit blocks outward
    /// Only for use when setting/removing a block
    // Based on https://www.seedofandromeda.com/blogs/29-fast-flood-fill-lighting-in-a-blocky-voxel-game-pt-1
    fn propagate_sunlight_queue(
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
            if z >= 0 && z < WORLD_HEIGHT_CHUNKS as i16 {
                let cp = ChunkPos::from_world_block_coords(x, y, z);
                let chunk = self.get_chunk_pos(cp).unwrap();
                let light_level = chunk.get_block_world(x, y, z).get_sunlight();
                sunlight_helper(self, x + 1, y, z, light_level, dirty_chunks, &mut queue);
                sunlight_helper(self, x - 1, y, z, light_level, dirty_chunks, &mut queue);
                sunlight_helper(self, x, y + 1, z, light_level, dirty_chunks, &mut queue);
                sunlight_helper(self, x, y - 1, z, light_level, dirty_chunks, &mut queue);
                if z < WORLD_HEIGHT_BLOCKS - 1 {
                    sunlight_helper(self, x, y, z + 1, light_level, dirty_chunks, &mut queue);
                }
                if z > 0 {
                    sunlight_helper(self, x, y, z - 1, light_level, dirty_chunks, &mut queue);
                }
            }
        }
    }

    // Remove sunlight from this block and also remove sunlight propagated from here
    fn remove_sunlight_if_needed(
        &mut self,
        wbx: i16,
        wby: i16,
        wbz: i16,
        old_block_sunlight: u8,
        dirty_chunks: &mut HashSet<ChunkPos>,
    ) {
        let mut queue = Vec::new();
        queue.push((wbx, wby, wbz, old_block_sunlight));
        if old_block_sunlight == MAX_LIGHT_LEVEL {
            // The old block was in direct sunlight, remove propagation downwards
            for z in (0..wbz).rev() {
                let mut block = self.get_block(wbx, wby, z);
                if block.is_opaque() {
                    break;
                } else {
                    queue.push((wbx, wby, z, block.get_sunlight()));
                    block.set_sunlight(0);
                    self.set_block(wbx, wby, z, block);
                    dirty_chunks.insert(ChunkPos::from_world_block_coords(wbx, wby, z));
                }
            }
        }
        remove_sunlight_queue(self, queue, dirty_chunks);
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

// Helper for propagate_sunlight_queue that propagates sunlight 1 block
fn sunlight_helper(
    chunks: &mut ChunkBuffer,
    x: i16,
    y: i16,
    z: i16,
    light_level: u8,
    dirty_chunks: &mut HashSet<ChunkPos>,
    queue: &mut Vec<(i16, i16, i16)>,
) {
    let cp = ChunkPos::from_world_block_coords(x, y, z);
    let chunk = chunks.get_mut_chunk_pos(cp).unwrap();
    let lbx = (x - cp.x * CHUNK_SIZE as i16) as usize;
    let lby = (y - cp.y * CHUNK_SIZE as i16) as usize;
    let lbz = (z - cp.z * CHUNK_SIZE as i16) as usize;
    let mut block = chunk.get_block(lbx, lby, lbz);
    let block_light = block.get_sunlight();
    if block.is_transparent() && block_light + 2 <= light_level {
        block.set_sunlight(light_level - 1);
        chunk.set_block(lbx, lby, lbz, block);
        dirty_chunks.insert(cp);
        queue.push((x, y, z));
    }
}

// Remove sunlight previously propagated from the blocks in this queue.
fn remove_sunlight_queue(
    chunks: &mut ChunkBuffer,
    start_queue: Vec<(i16, i16, i16, u8)>,
    dirty_chunks: &mut HashSet<ChunkPos>,
) {
    let mut queue = VecDeque::new();
    let mut propagate_queue = Vec::new();
    for q in start_queue {
        queue.push_back(q);
    }
    // Remove all light from this source
    while !queue.is_empty() {
        let (wbx, wby, wbz, sunlight_level) = queue.pop_front().unwrap();
        sunlight_removal_helper(
            chunks,
            wbx - 1,
            wby,
            wbz,
            sunlight_level,
            dirty_chunks,
            &mut queue,
            &mut propagate_queue,
        );
        sunlight_removal_helper(
            chunks,
            wbx + 1,
            wby,
            wbz,
            sunlight_level,
            dirty_chunks,
            &mut queue,
            &mut propagate_queue,
        );
        sunlight_removal_helper(
            chunks,
            wbx,
            wby - 1,
            wbz,
            sunlight_level,
            dirty_chunks,
            &mut queue,
            &mut propagate_queue,
        );
        sunlight_removal_helper(
            chunks,
            wbx,
            wby + 1,
            wbz,
            sunlight_level,
            dirty_chunks,
            &mut queue,
            &mut propagate_queue,
        );
        sunlight_removal_helper(
            chunks,
            wbx,
            wby,
            wbz + 1,
            sunlight_level,
            dirty_chunks,
            &mut queue,
            &mut propagate_queue,
        );
        sunlight_removal_helper(
            chunks,
            wbx,
            wby,
            wbz - 1,
            sunlight_level,
            dirty_chunks,
            &mut queue,
            &mut propagate_queue,
        );
    }
    // Propagate back sunlight from adjacent blocks
    chunks.propagate_sunlight_queue(propagate_queue, dirty_chunks);
}

fn sunlight_removal_helper(
    chunks: &mut ChunkBuffer,
    wbx: i16,
    wby: i16,
    wbz: i16,
    sunlight_level: u8,
    dirty_chunks: &mut HashSet<ChunkPos>,
    queue: &mut VecDeque<(i16, i16, i16, u8)>,
    propagate_queue: &mut Vec<(i16, i16, i16)>,
) {
    let cp = ChunkPos::from_world_block_coords(wbx, wby, wbz);
    if cp.z < 0 || cp.z >= WORLD_HEIGHT_CHUNKS as i16 {
        return;
    }
    let chunk = chunks.get_mut_chunk_pos(cp).unwrap();
    let lbx = (wbx - cp.x * CHUNK_SIZE as i16) as usize;
    let lby = (wby - cp.y * CHUNK_SIZE as i16) as usize;
    let lbz = (wbz - cp.z * CHUNK_SIZE as i16) as usize;
    let mut block = chunk.get_block(lbx, lby, lbz);
    if block.is_transparent() {
        let block_sunlight = block.get_sunlight();
        if block_sunlight != 0 && block_sunlight < sunlight_level {
            block.set_sunlight(0);
            assert!(block.get_sunlight() == 0);
            chunk.set_block(lbx, lby, lbz, block);
            dirty_chunks.insert(cp);
            queue.push_back((wbx, wby, wbz, block_sunlight));
        } else if block_sunlight > sunlight_level {
            propagate_queue.push((wbx, wby, wbz));
        }
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
        // TODO neighbouring chunks may be affected by this block (light) and may need to be remeshed
        dirty_chunks.insert(cp);
        queue.push((x, y, z));
    }
}
