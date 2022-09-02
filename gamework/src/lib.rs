mod app;
mod assets;
// pub mod audio;
// pub mod egui_glow;
mod flat_grid;
mod gui;
mod input;
pub mod profile;
mod state;
mod system_context;
pub mod video;

extern crate nalgebra_glm as glm;

pub use app::App;
pub use assets::Assets;
pub use flat_grid::*;
pub use glow;
pub use gui::*;
pub use input::*;
use noise::*;
pub use state::*;
pub use system_context::{SystemContext, PROFILE_SAMPLES};

pub trait NoiseHelpers {
    fn noise_2d(&self, x: f32, y: f32, scale: f32) -> f32;
    fn noise_2d_default(&self, x: f32, y: f32, scale: f32) -> f32;
}

impl NoiseHelpers for Perlin {
    fn noise_2d(&self, x: f32, y: f32, scale: f32) -> f32 {
        (self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32)
            * 0.5
            + 0.5
    }

    fn noise_2d_default(&self, x: f32, y: f32, scale: f32) -> f32 {
        self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32
    }
}

impl NoiseHelpers for Fbm {
    fn noise_2d(&self, x: f32, y: f32, scale: f32) -> f32 {
        (self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32)
            * 0.5
            + 0.5
    }

    fn noise_2d_default(&self, x: f32, y: f32, scale: f32) -> f32 {
        self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32
    }
}

impl NoiseHelpers for Billow {
    fn noise_2d(&self, x: f32, y: f32, scale: f32) -> f32 {
        (self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32)
            * 0.5
            + 0.5
    }

    fn noise_2d_default(&self, x: f32, y: f32, scale: f32) -> f32 {
        self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32
    }
}

impl NoiseHelpers for HybridMulti {
    fn noise_2d(&self, x: f32, y: f32, scale: f32) -> f32 {
        (self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32)
            * 0.5
            + 0.5
    }

    fn noise_2d_default(&self, x: f32, y: f32, scale: f32) -> f32 {
        self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32
    }
}

impl NoiseHelpers for RidgedMulti {
    fn noise_2d(&self, x: f32, y: f32, scale: f32) -> f32 {
        (self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32)
            * 0.5
            + 0.5
    }

    fn noise_2d_default(&self, x: f32, y: f32, scale: f32) -> f32 {
        self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32
    }
}

impl NoiseHelpers for Value {
    fn noise_2d(&self, x: f32, y: f32, scale: f32) -> f32 {
        (self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32)
            * 0.5
            + 0.5
    }

    fn noise_2d_default(&self, x: f32, y: f32, scale: f32) -> f32 {
        self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32
    }
}

impl NoiseHelpers for Worley {
    fn noise_2d(&self, x: f32, y: f32, scale: f32) -> f32 {
        (self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32)
            * 0.5
            + 0.5
    }

    fn noise_2d_default(&self, x: f32, y: f32, scale: f32) -> f32 {
        self.get([
            x as f64 * scale as f64 + 0.1234,
            y as f64 * scale as f64 + 0.5678,
        ]) as f32
    }
}
