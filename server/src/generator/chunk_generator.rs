use crate::generator::generators::*;
use crate::object_placer::ObjectPlacer;
use crate::object_placer::TreePlacer;
use common::block::*;
use common::chunk::*;
use common::world_type::GeneratorType;
use std::i16;

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
