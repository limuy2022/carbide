use crate::control_flow;

use super::{Event, ParseControlFlow};

pub struct Keyboard {
    state: State,
}

#[derive(Clone, Debug)]
pub struct Key {
    pub char: u8,
    pub modifiers: KeyModifiers,
}

#[derive(Clone, Debug, Default)]
pub struct KeyModifiers {
    pub alt: bool,
    pub meta: bool,
    pub shift: bool,
    pub control: bool,
}

enum State {
    Separator,
    Modifier(u8),
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            state: State::Separator,
        }
    }
}

impl Keyboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn key(key: u8, modifiers: u8) -> Option<Event> {
        let modifiers = KeyModifiers::parse(modifiers);
        let char = match key {
            // Up
            b'A' => 0x11,
            // Down
            b'B' => 0x12,
            // Right
            b'C' => 0x13,
            // Left
            b'D' => 0x14,
            _ => return None,
        };

        Some(Event::KeyPress {
            key: Key { char, modifiers },
        })
    }

    pub fn parse(&mut self, key: u8) -> ParseControlFlow {
        self.state = match self.state {
            State::Separator => match key {
                b';' => State::Modifier(0),
                _ => control_flow!(break)?,
            },
            State::Modifier(code) => match key {
                b'0'..=b'9' => State::Modifier(code * 10 + key - b'0'),
                key => control_flow!(break Self::key(key, code))?,
            },
        };

        control_flow!(continue)
    }
}

impl From<u8> for Key {
    fn from(char: u8) -> Self {
        Self {
            char,
            modifiers: KeyModifiers::default(),
        }
    }
}

impl KeyModifiers {
    pub fn parse(key: u8) -> Self {
        let (alt, meta, shift, control) = (0b1000, 0b0100, 0b0010, 0b0001);
        let mask = match key {
            2 => shift,
            3 => alt,
            4 => shift | alt,
            5 => control,
            6 => shift | control,
            7 => alt | control,
            8 => shift | alt | control,
            9 => meta,
            10 => meta | shift,
            11 => meta | alt,
            12 => meta | alt | shift,
            13 => meta | control,
            14 => meta | control | shift,
            15 => meta | control | alt,
            16 => meta | control | alt | shift,
            _ => 0,
        };

        KeyModifiers {
            alt: alt & mask != 0,
            meta: meta & mask != 0,
            shift: shift & mask != 0,
            control: control & mask != 0,
        }
    }
}
