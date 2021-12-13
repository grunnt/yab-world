use crate::chunk::*;
use nalgebra_glm::Vec3;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct ChunkPos {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl ChunkPos {
    pub fn zero() -> ChunkPos {
        ChunkPos { x: 0, y: 0, z: 0 }
    }

    pub fn new(x: i16, y: i16, z: i16) -> ChunkPos {
        ChunkPos { x, y, z }
    }

    pub fn from_world_pos(pos: Vec3) -> ChunkPos {
        let mut cp = ChunkPos { x: 0, y: 0, z: 0 };
        cp.set_from_world_pos(pos);
        cp
    }

    pub fn from_world_block_coords(wbx: i16, wby: i16, wbz: i16) -> ChunkPos {
        ChunkPos {
            x: wbx >> 4,
            y: wby >> 4,
            z: wbz >> 4,
        }
    }

    pub fn set_from_world_pos(&mut self, pos: Vec3) {
        self.x = (pos.x / CHUNK_SIZE as f32).floor() as i16;
        self.y = (pos.y / CHUNK_SIZE as f32).floor() as i16;
        self.z = (pos.z / CHUNK_SIZE as f32).floor() as i16;
    }

    pub fn set_from_world_coords(&mut self, wx: f32, wy: f32, wz: f32) {
        self.x = (wx / CHUNK_SIZE as f32).floor() as i16;
        self.y = (wy / CHUNK_SIZE as f32).floor() as i16;
        self.z = (wz / CHUNK_SIZE as f32).floor() as i16;
    }

    pub fn dist_squared_from(&self, other: &ChunkPos) -> i16 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn xp(&self) -> ChunkPos {
        ChunkPos {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }
    pub fn xm(&self) -> ChunkPos {
        ChunkPos {
            x: self.x - 1,
            y: self.y,
            z: self.z,
        }
    }
    pub fn yp(&self) -> ChunkPos {
        ChunkPos {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }
    pub fn ym(&self) -> ChunkPos {
        ChunkPos {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        }
    }
    pub fn zp(&self) -> ChunkPos {
        ChunkPos {
            x: self.x,
            y: self.y,
            z: self.z + 1,
        }
    }
    pub fn zm(&self) -> ChunkPos {
        ChunkPos {
            x: self.x,
            y: self.y,
            z: self.z - 1,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct ChunkRegionPos {
    pub x: i16,
    pub y: i16,
}

impl ChunkRegionPos {
    pub fn new(x: i16, y: i16) -> Self {
        ChunkRegionPos { x, y }
    }

    pub fn from_world_pos(wx: i16, wy: i16) -> Self {
        ChunkRegionPos {
            x: wx.div_euclid(REGION_SIZE_BLOCKS),
            y: wy.div_euclid(REGION_SIZE_BLOCKS),
        }
    }

    pub fn world_x(&self) -> i16 {
        self.x * REGION_SIZE_BLOCKS
    }

    pub fn world_y(&self) -> i16 {
        self.y * REGION_SIZE_BLOCKS
    }

    pub fn contains_world_pos(&self, wx: i16, wy: i16) -> bool {
        let region = ChunkRegionPos::from_world_pos(wx, wy);
        *self == region
    }
}

#[cfg(test)]
mod chunk_pos_test {

    use crate::chunk::ChunkPos;

    #[test]
    fn from_world_block_coords() {
        assert_eq!(
            ChunkPos::from_world_block_coords(-132, -132, -132),
            ChunkPos::new(-9, -9, -9)
        );

        assert_eq!(
            ChunkPos::from_world_block_coords(-16, -16, -16),
            ChunkPos::new(-1, -1, -1)
        );

        assert_eq!(
            ChunkPos::from_world_block_coords(0, 0, 0),
            ChunkPos::new(0, 0, 0)
        );

        assert_eq!(
            ChunkPos::from_world_block_coords(1, 1, 1),
            ChunkPos::new(0, 0, 0)
        );

        assert_eq!(
            ChunkPos::from_world_block_coords(16, 16, 16),
            ChunkPos::new(1, 1, 1)
        );
    }
}
