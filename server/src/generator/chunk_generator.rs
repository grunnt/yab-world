use crate::generator::generators::*;
use crate::generator::object_placer::TreePlacer;
use common::block::*;
use common::chunk::*;
use common::world_type::GeneratorType;
use std::i16;

use super::PoiPlacer;

pub struct ColumnGenerator {
    hills_generator: HillsGenerator,
    flat_generator: FlatGenerator,
    water_generator: WaterWorldGenerator,
    alien_generator: AlienGenerator,
    tree_placer: TreePlacer,
    poi_placer: PoiPlacer,
}

impl ColumnGenerator {
    pub fn new(seed: u32) -> Self {
        ColumnGenerator {
            hills_generator: HillsGenerator::new(seed),
            flat_generator: FlatGenerator::new(32, 36),
            water_generator: WaterWorldGenerator::new(seed),
            alien_generator: AlienGenerator::new(seed),
            tree_placer: TreePlacer::new(seed),
            poi_placer: PoiPlacer::new(seed),
        }
    }

    pub fn generate_column(
        &mut self,
        world_type: GeneratorType,
        col: ChunkColumnPos,
    ) -> Vec<Chunk> {
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
                // Generate the terrain
                let generator: &mut dyn Generator = match world_type {
                    GeneratorType::Flat => &mut self.flat_generator,
                    GeneratorType::Water => &mut self.water_generator,
                    GeneratorType::Alien => &mut self.alien_generator,
                    GeneratorType::Default => &mut self.hills_generator,
                };
                let mut blocks = generator.generate(x, y);
                // Place trees and points of interest
                self.tree_placer.place(x, y, &mut blocks, generator);
                self.poi_placer.place(x, y, &mut blocks, generator);
                // Copy the results into the chunk column
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
