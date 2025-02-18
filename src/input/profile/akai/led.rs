#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ApcMiniMk2ButtonLedMode {
    IntensFull,
    Pulsing1o16,
    Pulsing1o8,
    Pulsing1o4,
    Pulsing1o2,
    Blinking1o24,
    Blinking1o16,
    Blinking1o8,
    Blinking1o4,
    Blinking1o2,
}

impl ApcMiniMk2ButtonLedMode {
    pub fn value(&self) -> u8 {
        match self {
            Self::IntensFull => 0x6,
            Self::Pulsing1o16 => 0x7,
            Self::Pulsing1o8 => 0x8,
            Self::Pulsing1o4 => 0x9,
            Self::Pulsing1o2 => 0xA,
            Self::Blinking1o24 => 0xB,
            Self::Blinking1o16 => 0xC,
            Self::Blinking1o8 => 0xD,
            Self::Blinking1o4 => 0xE,
            Self::Blinking1o2 => 0xF,
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            Self::IntensFull => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ApcMiniMk2ButtonLedColor {
    Off,

    White,
    Red,
    Green,
    Orange,
    Pink,
    Blue,
}

impl ApcMiniMk2ButtonLedColor {
    pub fn value(&self) -> u8 {
        match self {
            Self::Off => 0,
            Self::White => 3,
            Self::Red => 5,
            Self::Green => 21,
            Self::Orange => 9,
            Self::Pink => 106,
            Self::Blue => 45,
        }
    }
}
