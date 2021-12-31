mod tower_placer;
mod tree_placer;

use noise::{Perlin, Value};
pub use tower_placer::*;
pub use tree_placer::*;

use super::NoiseSource2D;

#[derive(Clone, Debug)]
pub struct WorldObject {
    center_x: i16,
    center_y: i16,
    size: i16,
    random: f64,
    density: f64,
}

pub struct ObjectGrid {
    grid_size: i16,
    min_object_size: i16,
    max_object_size: i16,
    object_density: f64,
    grid_x_noise: NoiseSource2D<Value>,
    grid_y_noise: NoiseSource2D<Value>,
    density_noise: NoiseSource2D<Perlin>,
    randomizer_noise: NoiseSource2D<Value>,
}

impl ObjectGrid {
    pub fn new(
        seed: u32,
        grid_size: i16,
        min_object_size: i16,
        max_object_size: i16,
        object_density: f64,
    ) -> Self {
        let grid_size = grid_size.max(max_object_size * 2);
        ObjectGrid {
            grid_size,
            min_object_size,
            max_object_size,
            object_density,
            grid_x_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 1.0),
            grid_y_noise: NoiseSource2D::<Value>::new_value(seed, 0.0, 1.0),
            density_noise: NoiseSource2D::<Perlin>::new_perlin(seed, 0.0, 1.0),
            randomizer_noise: NoiseSource2D::<Value>::new_value(seed, 1.0, std::i16::MAX as f64),
        }
    }

    pub fn try_get_object(&self, x: i16, y: i16) -> Option<WorldObject> {
        let grid_x = (x / self.grid_size) * self.grid_size;
        let grid_y = (y / self.grid_size) * self.grid_size;
        let random = self.randomizer_noise.get(grid_x as f64, grid_y as f64, 1.0);
        let size =
            self.min_object_size + (random as i16) % (self.max_object_size - self.min_object_size);
        let grid_range = (self.grid_size - size) as f64;
        let center_x = grid_x
            + size / 2
            + (self
                .grid_x_noise
                .get(grid_x as f64 + 0.123, grid_y as f64 + 50.665, 1.0)
                * grid_range) as i16;
        let center_y = grid_y
            + size / 2
            + (self
                .grid_y_noise
                .get(grid_x as f64 - 102.4, grid_y as f64 + 553.1, 1.0)
                * grid_range) as i16;
        let density = self
            .density_noise
            .get(center_x as f64, center_y as f64, 0.01);
        let half_size = size / 2;
        if density < self.object_density
            && x >= center_x - half_size
            && x <= center_x + half_size
            && y >= center_y - half_size
            && y <= center_y + half_size
        {
            return Some(WorldObject {
                center_x,
                center_y,
                size,
                random,
                density,
            });
        }

        None
    }
}
