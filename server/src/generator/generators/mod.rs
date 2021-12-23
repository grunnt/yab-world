mod alien_generator;
mod flat_generator;
mod hills_generator;
mod water_generator;

pub use alien_generator::AlienGenerator;
use common::block::Block;
pub use flat_generator::FlatGenerator;
pub use hills_generator::HillsGenerator;
pub use water_generator::WaterWorldGenerator;

pub trait Generator {
    fn generate(&mut self, x: i16, y: i16) -> Vec<Block>;
}
