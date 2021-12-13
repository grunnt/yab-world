use crate::Position;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}
#[derive(Clone, Debug)]
pub enum InputEvent {
    MouseMove { x: f32, y: f32, dx: f32, dy: f32 },
    MouseOn,
    MouseOff,
    MouseClick { x: f32, y: f32, button: MouseButton },
    MouseScroll { delta: f32 },
    KeyPress { key: Key, shift: bool },
}

impl InputEvent {
    pub fn try_get_position(&self) -> Option<Position> {
        match self {
            InputEvent::MouseMove { x, y, .. } => Some(Position::new(*x, *y)),
            InputEvent::MouseClick { x, y, .. } => Some(Position::new(*x, *y)),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Period,
    Comma,
    Space,
    Backspace,
    Tab,
    LCtrl,
    RCtrl,
    LShift,
    RShift,
    Escape,
    Up,
    Right,
    Down,
    Left,
}

impl Key {
    pub fn to_char(&self) -> Option<char> {
        match self {
            Key::A => Some('A'),
            Key::B => Some('B'),
            Key::C => Some('C'),
            Key::D => Some('D'),
            Key::E => Some('E'),
            Key::F => Some('F'),
            Key::G => Some('G'),
            Key::H => Some('H'),
            Key::I => Some('I'),
            Key::J => Some('J'),
            Key::K => Some('K'),
            Key::L => Some('L'),
            Key::M => Some('M'),
            Key::N => Some('N'),
            Key::O => Some('O'),
            Key::P => Some('P'),
            Key::Q => Some('Q'),
            Key::R => Some('R'),
            Key::S => Some('S'),
            Key::T => Some('T'),
            Key::U => Some('U'),
            Key::V => Some('V'),
            Key::W => Some('W'),
            Key::X => Some('X'),
            Key::Y => Some('Y'),
            Key::Z => Some('Z'),
            Key::Num0 => Some('0'),
            Key::Num1 => Some('1'),
            Key::Num2 => Some('2'),
            Key::Num3 => Some('3'),
            Key::Num4 => Some('4'),
            Key::Num5 => Some('5'),
            Key::Num6 => Some('6'),
            Key::Num7 => Some('7'),
            Key::Num8 => Some('8'),
            Key::Num9 => Some('9'),
            Key::Period => Some('.'),
            Key::Comma => Some(','),
            Key::Space => Some(' '),
            _ => None,
        }
    }
}
