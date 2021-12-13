use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRGBA {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> ColorRGBA {
        ColorRGBA { r, g, b, a }
    }

    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> ColorRGBA {
        ColorRGBA {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0,
            a: (a as f32) / 255.0,
        }
    }

    pub fn white() -> ColorRGBA {
        ColorRGBA::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn black() -> ColorRGBA {
        ColorRGBA::new(0.0, 0.0, 0.0, 1.0)
    }
}

#[derive(Clone, Debug)]
pub struct ColorRGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl ColorRGB {
    pub fn new(r: f32, g: f32, b: f32) -> ColorRGB {
        ColorRGB { r, g, b }
    }

    pub fn to_rgba(&self, a: f32) -> ColorRGBA {
        ColorRGBA {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorRGBu8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorRGBu8 {
    pub fn new(r: u8, g: u8, b: u8) -> ColorRGBu8 {
        ColorRGBu8 { r, g, b }
    }

    pub fn pack_as_rgb_u32(&self) -> u32 {
        (self.r as u32) | (self.g as u32) << 8 | (self.b as u32) << 16
    }

    pub fn to_rgba(&self, a: f32) -> ColorRGBA {
        ColorRGBA::new(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            a,
        )
    }
}
