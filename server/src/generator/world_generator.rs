use common::block::*;
use common::chunk::*;
use common::world_type::GeneratorType;
use crossbeam::channel::*;
use crossbeam::unbounded;
use log::*;
use std::sync::Arc;
use std::thread;

use crate::generator::column_generator::ColumnGenerator;
use crate::generator::ObjectGenerator;
use crate::generator::PregeneratedObject;
use crate::generator::TowerGenerator;

pub struct WorldGenerator {
    pub worker_count: usize,
    colpos_tx: Sender<ChunkColumnPos>,
    column_rx: Receiver<(ChunkColumnPos, Vec<Chunk>)>,
}

impl WorldGenerator {
    pub fn new(seed: u32, world_type: GeneratorType) -> WorldGenerator {
        // let seed = 1234;
        let poi_object_list = {
            let mut result = Vec::new();
            let mut gen = TowerGenerator::new(seed);
            for _ in 0..5 {
                result.push(gen.generate());
            }
            Arc::new(result)
        };
        let tree_object_list = Arc::new(vec![PregeneratedObject::solid(
            1,
            1,
            16,
            Block::log_block(),
            Block::dirt_block(),
        )]);
        let worker_count = num_cpus::get() - 1;
        info!("Initializing {} generator workers", worker_count);
        // Start chunk column generator threads
        let (colpos_tx, colpos_rx) = unbounded();
        let (column_tx, column_rx) = unbounded();
        for id in 0..worker_count {
            let colpos_rx = colpos_rx.clone();
            let column_tx = column_tx.clone();
            let poi_object_list = Arc::clone(&poi_object_list);
            let tree_object_list = Arc::clone(&tree_object_list);
            thread::Builder::new()
                .name(format!("generator{}", id).to_string())
                .spawn(move || {
                    let mut generator =
                        ColumnGenerator::new(seed, poi_object_list, tree_object_list);
                    info!("Starting generator {}", id);
                    loop {
                        match colpos_rx.recv() {
                            Ok(col) => {
                                let column = generator.generate_column(world_type, col);
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
    pub fn generate(&mut self, col: ChunkColumnPos) {
        self.colpos_tx.send(col).unwrap();
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
