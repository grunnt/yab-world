use noise::*;

pub struct NoiseSource2D<N>
where
    N: NoiseFn<[f64; 2]>,
{
    noise: N,
    min: f64,
    max: f64,
    x_offset: f64,
    y_offset: f64,
}

impl<N> NoiseSource2D<N>
where
    N: NoiseFn<[f64; 2]>,
{
    pub fn new_perlin(seed: u32, min: f64, max: f64) -> NoiseSource2D<Perlin> {
        NoiseSource2D {
            noise: Perlin::new().set_seed(seed),
            min,
            max,
            x_offset: 123.123,
            y_offset: 234.234,
        }
    }

    pub fn new_fbm(seed: u32, min: f64, max: f64) -> NoiseSource2D<Fbm> {
        NoiseSource2D {
            noise: Fbm::new().set_seed(seed),
            min,
            max,
            x_offset: 321.321,
            y_offset: 432.432,
        }
    }

    pub fn new_value(seed: u32, min: f64, max: f64) -> NoiseSource2D<Value> {
        NoiseSource2D {
            noise: Value::new().set_seed(seed),
            min,
            max,
            x_offset: 632.641,
            y_offset: 12.692,
        }
    }

    pub fn get(&self, x: f64, y: f64, scale: f64) -> f64 {
        let noise = self
            .noise
            .get([(x + self.x_offset) * scale, (y + self.y_offset) * scale])
            * 0.5
            + 0.5;
        (self.min + noise * (self.max - self.min)).clamp(self.min, self.max)
    }
}

pub struct NoiseSource3D<N>
where
    N: NoiseFn<[f64; 3]>,
{
    noise: N,
    min: f64,
    max: f64,
    x_offset: f64,
    y_offset: f64,
    z_offset: f64,
}

impl<N> NoiseSource3D<N>
where
    N: NoiseFn<[f64; 3]>,
{
    pub fn new_perlin(seed: u32, min: f64, max: f64) -> NoiseSource3D<Perlin> {
        NoiseSource3D {
            noise: Perlin::new().set_seed(seed),
            min,
            max,
            x_offset: 123.123,
            y_offset: 234.234,
            z_offset: 345.345,
        }
    }

    pub fn new_fbm(seed: u32, min: f64, max: f64) -> NoiseSource3D<Fbm> {
        NoiseSource3D {
            noise: Fbm::new().set_seed(seed),
            min,
            max,
            x_offset: 321.321,
            y_offset: 432.432,
            z_offset: 543.543,
        }
    }

    pub fn get(&self, x: f64, y: f64, z: f64, scale: f64) -> f64 {
        let noise = self.noise.get([
            (x + self.x_offset) * scale,
            (y + self.y_offset) * scale,
            (z + self.z_offset) * scale,
        ]) * 0.5
            + 0.5;
        (self.min + noise * (self.max - self.min)).clamp(self.min, self.max)
    }
}
