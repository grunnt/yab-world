use common::chunk::*;
use common::comms::read_from::ReadFrom;
use common::comms::write_to::WriteTo;
use snap::read::FrameDecoder;
use snap::write::FrameEncoder;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::Instant;

pub struct SuperChunk {
    pub last_touched: Instant,
    pub chunk_data: HashMap<ChunkColumnPos, Vec<Vec<u8>>>,
}

impl SuperChunk {
    pub fn new() -> SuperChunk {
        SuperChunk {
            last_touched: Instant::now(),
            chunk_data: HashMap::new(),
        }
    }

    /// Save the superchunk
    pub fn save(&self, sc_path: &Path) {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(sc_path)
            .unwrap();
        let writer = BufWriter::new(file);
        let mut encoder = FrameEncoder::new(writer);
        // Write chunk count
        assert!(self.chunk_data.len() < std::u16::MAX as usize);
        (self.chunk_data.len() as u16)
            .write_to(&mut encoder)
            .unwrap();
        // Write chunk data
        for (cp, column_bytes) in &self.chunk_data {
            cp.x.write_to(&mut encoder).unwrap();
            cp.y.write_to(&mut encoder).unwrap();
            assert!(column_bytes.len() == WORLD_HEIGHT_CHUNKS);
            for bytes in column_bytes {
                assert!(bytes.len() < std::u16::MAX as usize);
                (bytes.len() as u16).write_to(&mut encoder).unwrap();
                encoder.write_all(&bytes).unwrap();
            }
        }
        encoder.flush().unwrap();
    }

    /// Load a superchunk
    pub fn load(sc_path: &Path) -> SuperChunk {
        let mut sc = SuperChunk::new();
        let file = File::open(sc_path).unwrap();
        let reader = BufReader::new(file);
        let mut decoder = FrameDecoder::new(reader);
        let chunk_count = u16::read_from(&mut decoder).unwrap();
        for _ in 0..chunk_count {
            let x = i16::read_from(&mut decoder).unwrap();
            let y = i16::read_from(&mut decoder).unwrap();
            let mut column_bytes = Vec::new();
            for _ in 0..WORLD_HEIGHT_CHUNKS {
                let length = u16::read_from(&mut decoder).unwrap() as usize;
                let mut bytes = vec![0; length];
                decoder.read_exact(&mut bytes).unwrap();
                column_bytes.push(bytes);
            }
            sc.chunk_data.insert(ChunkColumnPos { x, y }, column_bytes);
        }
        sc
    }
}
