use crate::generator::*;
use crate::object_placer::ObjectPlacer;
use crate::object_placer::TreePlacer;
use common::block::*;
use common::chunk::*;
use common::world_type::GeneratorType;
use crossbeam::channel::*;
use crossbeam::unbounded;
use log::*;
use std::i16;
use std::thread;

#[derive(Clone, Copy)]
pub struct GenerateMarker {
    pub player_id: u8,
    pub position: ChunkColumnPos,
    pub chunk_range: usize,
}

pub struct WorldGenerator {
    pub worker_count: usize,
    colpos_tx: Sender<(GeneratorType, ChunkColumnPos)>,
    column_rx: Receiver<(ChunkColumnPos, Vec<Chunk>)>,
}

impl WorldGenerator {
    pub fn new(worker_count: usize, seed: u32) -> WorldGenerator {
        // let seed = 1234;
        info!("Initializing {} generator workers", worker_count);
        // Start chunk column generator threads
        let (colpos_tx, colpos_rx) = unbounded();
        let (column_tx, column_rx) = unbounded();
        for id in 0..worker_count {
            let colpos_rx = colpos_rx.clone();
            let column_tx = column_tx.clone();
            thread::Builder::new()
                .name(format!("generator{}", id).to_string())
                .spawn(move || {
                    let mut generator = ColumnGenerator::new(seed);
                    info!("Starting generator {}", id);
                    loop {
                        match colpos_rx.recv() {
                            Ok((gen_type, col)) => {
                                let column = generator.generate_column(gen_type, col);
                                match column_tx.send((col, column)) {
                                    Err(e) => {
                                        debug!("generator {} shutting down: {}", id, e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => {
                                debug!("generator {} shutting down: {}", id, e);
                                break;
                            }
                        }
                    }
                })
                .unwrap();
        }
        WorldGenerator {
            worker_count,
            colpos_tx,
            column_rx,
        }
    }

    /// Place new generator work in queue
    pub fn generate(&mut self, world_type: GeneratorType, col: ChunkColumnPos) {
        //  println!("Generate {:?}", col);
        self.colpos_tx.send((world_type, col)).unwrap();
    }

    // Receive newly generated columns
    pub fn try_receive(&mut self) -> Option<(ChunkColumnPos, Vec<Chunk>)> {
        let col_opt = self.column_rx.try_recv();
        if col_opt.is_ok() {
            let c = col_opt.unwrap();
            Some(c)
        } else {
            None
        }
    }
}

pub struct ColumnGenerator {
    hills_generator: HillsGenerator,
    flat_generator: FlatGenerator,
    water_generator: WaterWorldGenerator,
    alien_generator: AlienGenerator,
    object_placer: TreePlacer,
}

impl ColumnGenerator {
    pub fn new(seed: u32) -> Self {
        ColumnGenerator {
            hills_generator: HillsGenerator::new(seed),
            flat_generator: FlatGenerator::new(32, 36),
            water_generator: WaterWorldGenerator::new(seed),
            alien_generator: AlienGenerator::new(seed),
            object_placer: TreePlacer::new(seed),
        }
    }

    fn generate_column(&mut self, world_type: GeneratorType, col: ChunkColumnPos) -> Vec<Chunk> {
        let cwx = col.x * CHUNK_SIZE as i16;
        let cwy = col.y * CHUNK_SIZE as i16;
        // Create empty column first
        let mut column = Vec::new();
        for z in 0..WORLD_HEIGHT_CHUNKS {
            column.push(Chunk::new_solid(
                ChunkPos::new(col.x, col.y, z as i16),
                Block::empty_block(),
            ));
        }
        // Generate the column in 1x1 columns of world height
        for rel_x in 0..CHUNK_SIZE {
            for rel_y in 0..CHUNK_SIZE {
                let x = rel_x as i16 + cwx;
                let y = rel_y as i16 + cwy;
                let mut blocks = match world_type {
                    GeneratorType::Flat => self.flat_generator.generate(x, y),
                    GeneratorType::Water => self.water_generator.generate(x, y),
                    GeneratorType::Alien => self.alien_generator.generate(x, y),
                    GeneratorType::Default => self.hills_generator.generate(x, y),
                };
                self.object_placer.place(x, y, &mut blocks);
                for cz in 0..WORLD_HEIGHT_CHUNKS {
                    let chunk = column.get_mut(cz).unwrap();
                    let chunk_bottom_z = cz * CHUNK_SIZE;
                    for rel_z in 0..CHUNK_SIZE {
                        chunk.set_block(rel_x, rel_y, rel_z, blocks[chunk_bottom_z + rel_z]);
                    }
                }
            }
        }

        column
    }
}
