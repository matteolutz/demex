use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Copy,
    Clone,
    EguiProbe,
    Default,
)]
pub enum FixtureChannelType {
    #[default]
    Unused,

    Intensity,

    IntensityFine,

    Pan,
    PanFine,
    Tilt,
    TiltFine,
    PanTiltSpeed,

    Red,
    RedFine,
    Green,
    GreenFine,
    Blue,
    BlueFine,
    White,
    WhiteFine,

    ColorMacro,
    ColorMacroCrossfade,
    ColorTemp,
    ColorTint,

    Prism,
    PrismRotation,
    RotatingGobos,
    GoboRotation,
    Zoom,
    ZoomFine,

    Focus,
    FocusFine,

    Shutter,

    ToggleFlags(usize),
}

impl FixtureChannelType {
    pub fn short_name(&self) -> String {
        match self {
            Self::Unused => "X".to_string(),

            Self::Intensity => "Int".to_string(),

            Self::IntensityFine => "IntF".to_string(),

            Self::Pan => "Pa".to_string(),
            Self::PanFine => "PaF".to_string(),
            Self::Tilt => "Ti".to_string(),
            Self::TiltFine => "TiF".to_string(),
            Self::PanTiltSpeed => "PaTiSp".to_string(),

            Self::Red => "Re".to_string(),
            Self::RedFine => "ReF".to_string(),
            Self::Green => "Gr".to_string(),
            Self::GreenFine => "GrF".to_string(),
            Self::Blue => "Bl".to_string(),
            Self::BlueFine => "BlF".to_string(),
            Self::White => "Wh".to_string(),
            Self::WhiteFine => "WhF".to_string(),

            Self::ColorMacro => "CoMa".to_string(),
            Self::ColorMacroCrossfade => "CMCr".to_string(),
            Self::ColorTemp => "CoTe".to_string(),
            Self::ColorTint => "CoTi".to_string(),

            Self::Prism => "Pr".to_string(),
            Self::PrismRotation => "PrR".to_string(),
            Self::RotatingGobos => "RoGo".to_string(),
            Self::GoboRotation => "GoRo".to_string(),
            Self::Zoom => "Zo".to_string(),
            Self::ZoomFine => "ZoF".to_string(),

            Self::Focus => "Fo".to_string(),
            Self::FocusFine => "FoF".to_string(),

            Self::Shutter => "Sh".to_string(),

            Self::ToggleFlags(idx) => format!("TF{}", idx),
        }
    }
}
