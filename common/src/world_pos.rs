#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct WorldPos {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl WorldPos {
    pub fn zero() -> Self {
        WorldPos { x: 0, y: 0, z: 0 }
    }

    pub fn new(x: i16, y: i16, z: i16) -> WorldPos {
        WorldPos { x, y, z }
    }

    pub fn dist_squared_from(&self, other: &WorldPos) -> i16 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        dx * dx + dy * dy + dz * dz
    }
}
