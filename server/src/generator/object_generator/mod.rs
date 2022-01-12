mod tower_generator;
mod tree_generator;

use super::PregeneratedObject;
pub use tower_generator::TowerGenerator;
pub use tree_generator::TreeGenerator;

pub trait ObjectGenerator {
    fn generate(&mut self) -> PregeneratedObject;
}
