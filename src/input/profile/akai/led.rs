use itertools::Itertools;
use strum::{EnumIter, IntoEnumIterator};

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

/// Colors: https://forum.bome.com/uploads/default/original/2X/b/bec6ef1cad2b5f100babf5780609739a8aeee1cf.png
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum ApcMiniMk2ButtonLedColor {
    Off,

    White,

    Red,

    Orange,

    Yellow,

    Green,

    Teal,

    Blue,
    DarkBlue,

    Pink,

    DarkViolet,
}

impl ApcMiniMk2ButtonLedColor {
    pub fn try_from_color(value: egui::Color32) -> Option<Self> {
        let [r, g, b, _] = value.to_array();

        ApcMiniMk2ButtonLedColor::iter()
            .sorted_by_key(|color| {
                let color_rgb = color.rgb();
                // Euclidean distance
                (color_rgb[0] as i32 - r as i32).pow(2)
                    + (color_rgb[1] as i32 - g as i32).pow(2)
                    + (color_rgb[2] as i32 - b as i32).pow(2)
            })
            .next()
    }
}

impl ApcMiniMk2ButtonLedColor {
    pub fn rgb(&self) -> [u8; 3] {
        match self {
            Self::Off => [0, 0, 0],
            Self::White => [0xFF, 0xFF, 0xFF],
            Self::Red => [0xFF, 0x00, 0x00],
            Self::Green => [0x00, 0xFF, 0x00],
            Self::Orange => [0xFF, 0x54, 0x00],
            Self::Yellow => [0xFF, 0xFF, 0x00],
            Self::Pink => [0xFF, 0x00, 0xFF],
            Self::Blue => [0x00, 0x00, 0xFF],
            Self::DarkBlue => [0x00, 0x00, 0x59],
            Self::DarkViolet => [0x2, 0x00, 0x13],
            Self::Teal => [0x00, 0xFF, 0x99],
        }
    }

    pub fn value(&self) -> u8 {
        match self {
            Self::Off => 0,
            Self::White => 3,
            Self::Red => 5,
            Self::Green => 21,
            Self::Orange => 9,
            Self::Teal => 33,
            Self::Yellow => 13,
            Self::Pink => 53,
            Self::Blue => 45,
            Self::DarkBlue => 46,
            Self::DarkViolet => 59,
        }
    }
}
