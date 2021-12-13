use std::collections::HashSet;

use crate::*;
use glutin::event::VirtualKeyCode;

pub struct Input {
    pressed_keys: HashSet<Key>,
    mouse_position: Option<Position>,
    mouse_captured: bool,
    mouse_left_down: bool,
    mouse_middle_down: bool,
    mouse_right_down: bool,
}

impl Input {
    pub fn new() -> Self {
        Input {
            pressed_keys: HashSet::new(),
            mouse_position: None,
            mouse_captured: false,
            mouse_left_down: false,
            mouse_middle_down: false,
            mouse_right_down: false,
        }
    }

    pub fn set_mouse_captured(&mut self, capture: bool) {
        self.mouse_captured = capture;
    }

    pub fn get_mouse_captured(&self) -> bool {
        self.mouse_captured
    }

    pub fn get_mouse_position(&self) -> &Option<Position> {
        &self.mouse_position
    }

    pub fn set_mouse_position(&mut self, position: Position) {
        self.mouse_position = Some(position);
    }

    pub fn set_mouse_button_state(&mut self, button: MouseButton, down: bool) {
        match button {
            MouseButton::Left => self.mouse_left_down = down,
            MouseButton::Middle => self.mouse_middle_down = down,
            MouseButton::Right => self.mouse_right_down = down,
        }
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.mouse_left_down,
            MouseButton::Middle => self.mouse_middle_down,
            MouseButton::Right => self.mouse_right_down,
        }
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn set_key_pressed(&mut self, key: Key, pressed: bool) {
        if pressed {
            self.pressed_keys.insert(key);
        } else {
            self.pressed_keys.remove(&key);
        }
    }

    pub fn set_pressed_keys(&mut self, keys: HashSet<Key>) {
        self.pressed_keys = keys;
    }
}

pub fn code_to_key(code: &VirtualKeyCode) -> Option<Key> {
    match code {
        VirtualKeyCode::A => Some(Key::A),
        VirtualKeyCode::B => Some(Key::B),
        VirtualKeyCode::C => Some(Key::C),
        VirtualKeyCode::D => Some(Key::D),
        VirtualKeyCode::E => Some(Key::E),
        VirtualKeyCode::F => Some(Key::F),
        VirtualKeyCode::G => Some(Key::G),
        VirtualKeyCode::H => Some(Key::H),
        VirtualKeyCode::I => Some(Key::I),
        VirtualKeyCode::J => Some(Key::J),
        VirtualKeyCode::K => Some(Key::K),
        VirtualKeyCode::L => Some(Key::L),
        VirtualKeyCode::M => Some(Key::M),
        VirtualKeyCode::N => Some(Key::N),
        VirtualKeyCode::O => Some(Key::O),
        VirtualKeyCode::P => Some(Key::P),
        VirtualKeyCode::Q => Some(Key::Q),
        VirtualKeyCode::R => Some(Key::R),
        VirtualKeyCode::S => Some(Key::S),
        VirtualKeyCode::T => Some(Key::T),
        VirtualKeyCode::U => Some(Key::U),
        VirtualKeyCode::V => Some(Key::V),
        VirtualKeyCode::W => Some(Key::W),
        VirtualKeyCode::X => Some(Key::X),
        VirtualKeyCode::Y => Some(Key::Y),
        VirtualKeyCode::Z => Some(Key::Z),
        VirtualKeyCode::Key0 => Some(Key::Num0),
        VirtualKeyCode::Key1 => Some(Key::Num1),
        VirtualKeyCode::Key2 => Some(Key::Num2),
        VirtualKeyCode::Key3 => Some(Key::Num3),
        VirtualKeyCode::Key4 => Some(Key::Num4),
        VirtualKeyCode::Key5 => Some(Key::Num5),
        VirtualKeyCode::Key6 => Some(Key::Num6),
        VirtualKeyCode::Key7 => Some(Key::Num7),
        VirtualKeyCode::Key8 => Some(Key::Num8),
        VirtualKeyCode::Key9 => Some(Key::Num9),
        VirtualKeyCode::Period => Some(Key::Period),
        VirtualKeyCode::Comma => Some(Key::Comma),
        VirtualKeyCode::Space => Some(Key::Space),
        VirtualKeyCode::Back => Some(Key::Backspace),
        VirtualKeyCode::Tab => Some(Key::Tab),
        VirtualKeyCode::LControl => Some(Key::LCtrl),
        VirtualKeyCode::RControl => Some(Key::RCtrl),
        VirtualKeyCode::LShift => Some(Key::LShift),
        VirtualKeyCode::RShift => Some(Key::RShift),
        VirtualKeyCode::Escape => Some(Key::Escape),
        VirtualKeyCode::Up => Some(Key::Up),
        VirtualKeyCode::Right => Some(Key::Right),
        VirtualKeyCode::Down => Some(Key::Down),
        VirtualKeyCode::Left => Some(Key::Left),
        _ => None,
    }
}
