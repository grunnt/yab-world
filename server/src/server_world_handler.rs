use core::time;
use std::{collections::HashMap, thread::sleep};

use common::{
    block::Block,
    chunk::{
        chunk_buffer::ChunkBuffer, ChunkColumn, ChunkColumnPos, ChunkPos, ColumnStatus, CHUNK_SIZE,
        WORLD_HEIGHT_CHUNKS,
    },
    comms::RleEncode,
    world_type::GeneratorType,
};
use nalgebra_glm::Vec3;

use crate::{generator::WorldGenerator, world_store::WorldStore};

pub struct ServerWorldHandler {
    store: WorldStore,
    buffer: ChunkBuffer,
    generator: WorldGenerator,
    generate_requests: HashMap<ChunkColumnPos, usize>,
    max_outstanding_work: usize,
    outstanding_work: usize,
}

impl ServerWorldHandler {
    /// Create a new world
    pub fn new(seed: u32, description: &str, world_type: GeneratorType) -> Self {
        ServerWorldHandler {
            store: WorldStore::new(seed, description, world_type),
            buffer: ChunkBuffer::new(),
            generator: WorldGenerator::new(seed, world_type),
            generate_requests: HashMap::new(),
            max_outstanding_work: num_cpus::get() * 2,
            outstanding_work: 0,
        }
    }

    /// Load an existing world
    pub fn load(seed: u32) -> Self {
        let store = WorldStore::load(seed).unwrap();
        let world_type = store.world_def().world_type;
        ServerWorldHandler {
            store,
            buffer: ChunkBuffer::new(),
            generator: WorldGenerator::new(seed, world_type),
            generate_requests: HashMap::new(),
            max_outstanding_work: num_cpus::get() * 2,
            outstanding_work: 0,
        }
    }

    pub fn time_on_start(&self) -> f32 {
        self.store.world_def().gametime
    }

    pub fn get_top_z(&self, wbx: i16, wby: i16) -> i16 {
        self.buffer.get_top_z(wbx, wby)
    }

    pub fn get_block(&mut self, wbx: i16, wby: i16, wbz: i16) -> Block {
        self.buffer.get_block(wbx, wby, wbz)
    }

    pub fn prepare_spawn_area(&mut self, col: ChunkColumnPos, chunk_range: i16) {
        // Put the generator to work
        for cy in col.y - chunk_range..col.y + chunk_range {
            for cx in col.x - chunk_range..col.x + chunk_range {
                let gen_col = ChunkColumnPos::new(cx, cy);
                self.buffer.store_column(ChunkColumn::new(
                    gen_col,
                    ColumnStatus::Requested,
                    Vec::new(),
                ));
                self.outstanding_work += 1;
                self.generator.generate(gen_col);
            }
        }
        // And wait until the work is done
        let mut preload_count = (chunk_range * 2) * (chunk_range * 2);
        while preload_count > 0 {
            if let Some(_) = self.try_get_generated_column(false) {
                preload_count -= 1;
            } else {
                sleep(time::Duration::from_millis(100));
            }
        }
    }

    /// Change a block in the world
    pub fn set_block(&mut self, wbx: i16, wby: i16, wbz: i16, block: Block) {
        let cp = ChunkPos::from_world_pos(Vec3::new(wbx as f32, wby as f32, wbz as f32));
        if cp.z >= 0 && cp.z < WORLD_HEIGHT_CHUNKS as i16 {
            let col = ChunkColumnPos::from_chunk_pos(cp);
            if let Some(column) = self.buffer.get_mut_column(col.x, col.y) {
                let chunk = &mut column.chunks[cp.z as usize];
                chunk.set_block(
                    (wbx - cp.x * CHUNK_SIZE as i16) as usize,
                    (wby - cp.y * CHUNK_SIZE as i16) as usize,
                    (wbz - cp.z * CHUNK_SIZE as i16) as usize,
                    block,
                );
                self.store.enqueue_chunk_save(&chunk);
            }
        }
    }

    pub fn try_clone_stored_column(&self, col: ChunkColumnPos) -> Option<Vec<Vec<u8>>> {
        if let Some(column) = self.buffer.get_column_pos(&col) {
            if column.status() == ColumnStatus::Stored {
                let mut block_data = Vec::new();
                for chunk in &column.chunks {
                    let mut bytes = Vec::new();
                    chunk.blocks.rle_encode_to(&mut bytes).unwrap();
                    block_data.push(bytes);
                }
                return Some(block_data);
            }
        }
        None
    }

    pub fn place_generate_request(&mut self, col: ChunkColumnPos) {
        if !self.buffer.is_column_requested(&col) {
            *self.generate_requests.entry(col).or_insert(0) += 1;
        }
    }

    pub fn retract_generate_request(&mut self, col: ChunkColumnPos) {
        let mut remove = false;
        if let Some(priority) = self.generate_requests.get_mut(&col) {
            if *priority == 1 {
                remove = true;
            } else {
                *priority -= 1;
            }
        }
        if remove {
            self.generate_requests.remove(&col);
        }
    }

    /// Are there any newly generated columns to distribute?
    pub fn try_get_generated_column(
        &mut self,
        return_data: bool,
    ) -> Option<(ChunkColumnPos, Vec<Vec<u8>>)> {
        if let Some((col, chunks)) = self.generator.try_receive() {
            if let Some(column) = self.buffer.get_mut_column(col.x, col.y) {
                self.outstanding_work -= 1;
                // Store the new column in the in-memory buffer
                column.set_status(ColumnStatus::Stored);
                column.chunks = chunks;
                // Save the column to disk
                self.store.enqueue_column_save(col, &column.chunks);
                if return_data {
                    // Return the column rle-encoded for distribution to clients
                    let mut block_data = Vec::new();
                    for chunk in &column.chunks {
                        let mut bytes = Vec::new();
                        chunk.blocks.rle_encode_to(&mut bytes).unwrap();
                        block_data.push(bytes);
                    }
                    Some((col, block_data))
                } else {
                    Some((col, Vec::new()))
                }
            } else {
                panic!("Generated column not in cache: {:?}", col);
            }
        } else {
            None
        }
    }

    /// Update the world, including periodical autosave
    pub fn update(&mut self, gametime: f32) {
        while self.outstanding_work < self.max_outstanding_work {
            let mut highest_priority = 0;
            let mut highest_priority_col = None;
            for (col, priority) in &self.generate_requests {
                if *priority > highest_priority {
                    highest_priority = *priority;
                    highest_priority_col = Some(col.clone());
                }
            }
            if let Some(col) = highest_priority_col {
                self.buffer.store_column(ChunkColumn::new(
                    col,
                    ColumnStatus::Requested,
                    Vec::new(),
                ));
                self.outstanding_work += 1;
                self.generator.generate(col);
                self.generate_requests.remove(&col);
            } else {
                break;
            }
        }
        self.store.save_world_if_needed(false, gametime);
    }

    /// Save the world now, e.g. before shutdown
    pub fn save(&mut self, gametime: f32) {
        self.store.save_world_if_needed(true, gametime);
    }
}
