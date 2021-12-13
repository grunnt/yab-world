mod flat_generator;
mod hills_generator;
mod noise_source;
mod water_generator;

use common::block::Block;
pub use flat_generator::*;
pub use hills_generator::*;
pub use noise_source::*;
pub use water_generator::*;

pub trait Generator {
    fn generate(&mut self, x: i16, y: i16) -> Vec<Block>;
}
