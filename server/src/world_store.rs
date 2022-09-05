use crate::superchunk::SuperChunk;
use common::chrono::Utc;
use common::chunk::*;
use common::comms::*;
use common::world_definition::*;
use common::world_type::GeneratorType;
use log::*;
use std::io::Cursor;
use std::time::{Duration, Instant};
use std::{collections::HashMap, path::PathBuf};

const MAX_SC_CACHE_DURATION: Duration = Duration::from_secs(60 * 5);
const MIN_SAVE_INTERVAL: Duration = Duration::from_millis(5000);

pub struct WorldStore {
    last_save: Instant,
    sc_cache: HashMap<String, SuperChunk>,
    save_queue: HashMap<ChunkColumnPos, Vec<Vec<u8>>>,
    world_folder: PathBuf,
    world_def: WorldDef,
}

impl WorldStore {
    pub fn new(seed: u32, description: &str, world_type: GeneratorType) -> Self {
        let world_list = WorldsStore::new();
        let world_folder = world_list.get_world_path(seed);
        WorldStore {
            last_save: Instant::now(),
            sc_cache: HashMap::new(),
            save_queue: HashMap::new(),
            world_folder: world_folder.into(),
            world_def: world_list.create_new_world(seed, description, world_type),
        }
    }

    pub fn load(seed: u32) -> Option<Self> {
        let world_list = WorldsStore::new();
        let world_folder = world_list.get_world_path(seed);
        if let Some(world_def) = world_list.try_load_world(seed) {
            Some(WorldStore {
                last_save: Instant::now(),
                sc_cache: HashMap::new(),
                save_queue: HashMap::new(),
                world_folder: world_folder.into(),
                world_def,
            })
        } else {
            None
        }
    }

    pub fn world_def(&self) -> &WorldDef {
        &self.world_def
    }

    /// Enqueue an updated chunk for saving when save_world_if_needed is called
    /// This assumes the chunk is already in the superchunk cache
    pub fn enqueue_chunk_save(&mut self, chunk: &Chunk) {
        let col = ChunkColumnPos::from_chunk_pos(chunk.pos);
        // Use run-length encoding to save memory and disk space
        let mut bytes = Vec::new();
        chunk.blocks.rle_encode_to(&mut bytes).unwrap();
        if let Some(column_bytes) = self.save_queue.get_mut(&col) {
            // Column is already in save queue, so update it there
            column_bytes[chunk.pos.z as usize] = bytes;
        } else if let Some(sc) = self.sc_cache.get_mut(&col_to_sc_filename(col)) {
            // Get the column from the superchunk cache and enqueue it for update
            let column_bytes = sc.chunk_data.get_mut(&col).unwrap();
            column_bytes[chunk.pos.z as usize] = bytes;
            self.save_queue.insert(col, column_bytes.clone());
        }
    }

    /// Enqueue a generated chunk column for saving when save_world_if_needed is called
    pub fn enqueue_column_save(&mut self, col: ChunkColumnPos, column: &Vec<Chunk>) {
        assert!(column.len() == WORLD_HEIGHT_CHUNKS);
        let mut column_bytes = Vec::new();
        for chunk in column {
            let mut bytes = Vec::new();
            // Use run-length encoding to save memory and disk space
            chunk.blocks.rle_encode_to(&mut bytes).unwrap();
            column_bytes.push(bytes);
        }
        self.save_queue.insert(col, column_bytes);
    }

    /// Load a column of chunks from disk (if available)
    pub fn load_column(&mut self, col: ChunkColumnPos) -> Option<Vec<Chunk>> {
        let sc_filename = col_to_sc_filename(col);

        // Is the superchunk in the cache?
        let in_cache = self.sc_cache.contains_key(&sc_filename);
        if !in_cache {
            // Not in the cache, is it on the disk?
            let sc_path = self.world_folder.join(format!("{}.chk", sc_filename));
            if sc_path.exists() {
                let sc = SuperChunk::load(&sc_path);
                self.sc_cache.insert(sc_filename.clone(), sc);
            } else {
                // Not available for loading, needs to be generated
                return None;
            }
        }

        // The superchunk is in the cache now. See if it contains the requested chunk.
        let sc = self.sc_cache.get_mut(&sc_filename).unwrap();
        sc.last_touched = Instant::now();
        if sc.chunk_data.contains_key(&col) {
            // Decode the chunk column and return it
            let mut chunks = Vec::new();
            let mut z = 0;
            for chunk_data in sc.chunk_data.get(&col).unwrap() {
                let blocks = Vec::rle_decode_from(&mut Cursor::new(chunk_data)).unwrap();
                let chunk = Chunk {
                    pos: ChunkPos {
                        x: col.x,
                        y: col.y,
                        z: z,
                    },
                    blocks,
                };
                chunks.push(chunk);
                z = z + 1;
            }
            return Some(chunks);
        }

        // Not in the superchunk, needs to be generated
        None
    }

    /// Save enqueued chunks if needed (i.e. when there are too many or a time period has passed)
    pub fn save_world_if_needed(&mut self, force: bool, game_time: f32) {
        // Is it time to save chunks?
        if !force && Instant::now().duration_since(self.last_save) < MIN_SAVE_INTERVAL {
            return;
        }

        self.world_def.gametime = game_time;
        self.world_def.timestamp = Utc::now();
        self.world_def.save(&self.world_folder.join(WORLD_DEF_FILE));

        // Group chunks by superchunk
        let mut c_per_sc: HashMap<String, Vec<ChunkColumnPos>> = HashMap::new();
        for col in self.save_queue.keys() {
            c_per_sc
                .entry(col_to_sc_filename(*col))
                .or_insert(Vec::new())
                .push(*col);
        }

        // Now get each superchunk in turn and save the chunk data
        for (sc_filename, cp_list) in c_per_sc {
            let sc_path = self.world_folder.join(format!("{}.chk", sc_filename));
            let new_sc_path = self.world_folder.join(format!("{}.new", sc_filename));
            if self.sc_cache.contains_key(&sc_filename) {
                // Get from superchunk cache
                let mut sc = self.sc_cache.get_mut(&sc_filename).unwrap();
                sc.last_touched = Instant::now();
                // Update chunk data
                for cp in cp_list {
                    if let Some(bytes) = self.save_queue.get(&cp) {
                        sc.chunk_data.insert(cp, bytes.clone());
                    }
                }
                sc.save(&new_sc_path);
            } else {
                let mut sc = if sc_path.exists() {
                    // Load existing superchunk
                    SuperChunk::load(&sc_path)
                } else {
                    // Create new superchunk
                    SuperChunk::new()
                };
                sc.last_touched = Instant::now();
                // Update chunk data
                for cp in cp_list {
                    if let Some(bytes) = self.save_queue.get(&cp) {
                        sc.chunk_data.insert(cp, bytes.clone());
                    }
                }
                sc.save(&new_sc_path);
                // Store superchunk in cache
                self.sc_cache.insert(sc_filename, sc);
            }
            // Rename if succesful
            if let Err(e) = std::fs::rename(&new_sc_path, &sc_path) {
                error!(
                    "Error renaming superchunk from {} to {}: {}",
                    &sc_path.to_string_lossy(),
                    &new_sc_path.to_string_lossy(),
                    e
                );
            }
        }

        self.save_queue.clear();
        self.clean_cache();
        self.last_save = Instant::now();
    }

    /// Remove unused superchunks from memory
    fn clean_cache(&mut self) {
        let now = Instant::now();
        self.sc_cache
            .retain(|_, sc| now.duration_since(sc.last_touched) < MAX_SC_CACHE_DURATION);
    }
}

fn col_to_sc_filename(col: ChunkColumnPos) -> String {
    let sc_x = col.x >> 5;
    let sc_y = col.y >> 5;
    format!("s{}_{}", sc_x, sc_y)
}
