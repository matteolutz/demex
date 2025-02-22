#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ApcMiniMk2ButtonLedMode {
    Intens10,
    Intens25,
    Intens50,
    Intens65,
    Intens75,
    Intens90,
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
            Self::Intens10 => 0x0,
            Self::Intens25 => 0x1,
            Self::Intens50 => 0x2,
            Self::Intens65 => 0x3,
            Self::Intens75 => 0x4,
            Self::Intens90 => 0x5,
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
        matches!(
            self,
            Self::Intens10
                | Self::Intens25
                | Self::Intens50
                | Self::Intens65
                | Self::Intens75
                | Self::Intens90
                | Self::IntensFull
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ApcMiniMk2ButtonLedColor {
    Off,

    White,
    Red,
    Green,
    Orange,
    Yellow,
    Pink,
    Blue,
    DarkBlue,
    DarkViolet,
}

impl ApcMiniMk2ButtonLedColor {
    pub fn value(&self) -> u8 {
        match self {
            Self::Off => 0,
            Self::White => 3,
            Self::Red => 5,
            Self::Green => 21,
            Self::Orange => 9,
            Self::Yellow => 13,
            Self::Pink => 106,
            Self::Blue => 45,
            Self::DarkBlue => 112,
            Self::DarkViolet => 59,
        }
    }
}
