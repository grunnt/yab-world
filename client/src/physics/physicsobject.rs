use glm::*;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum PhysicsObjectState {
    Stopped,
    Moving,
    Flying,
}

pub struct PhysicsObject {
    pub position: Vec3,
    pub velocity: Vec3,
    pub facing: Vec3,
    pub size: Vec3,
    pub state: PhysicsObjectState,
    pub on_ground: bool,
    pub was_in_water: bool,
    pub in_water: bool,
    pub controls: PhysicsObjectControls,
    pub gravity_factor: f32,
    pub colliding: bool, // Does the object collide with other objects or the ground?
}

impl PhysicsObject {}

pub struct PhysicsObjectControls {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub backward: bool,
    pub up: bool,
    pub down: bool,
    pub slower: bool,
}

impl PhysicsObjectControls {
    pub fn new() -> PhysicsObjectControls {
        PhysicsObjectControls {
            left: false,
            right: false,
            forward: false,
            backward: false,
            up: false,
            down: false,
            slower: false,
        }
    }

    pub fn is_moving(&self) -> bool {
        self.left || self.right || self.forward || self.backward
    }

    pub fn is_jumping(&self) -> bool {
        false // self.up
    }

    pub fn is_ducking(&self) -> bool {
        self.down
    }
}
