use common::chunk::*;
use common::world_type::GeneratorType;
use crossbeam::channel::*;
use crossbeam::unbounded;
use log::*;
use std::thread;

use crate::generator::chunk_generator::ColumnGenerator;

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
