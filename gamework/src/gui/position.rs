#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    } 

    pub fn zero() -> Self {
        Position { x: 0.0, y: 0.0 }
    }
}
