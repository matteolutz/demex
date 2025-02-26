use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use super::feature::feature_group::DefaultFeatureGroup;

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
    EnumIter,
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

    pub fn get_fine(&self) -> Option<Self> {
        match self {
            Self::Intensity => Some(Self::IntensityFine),
            Self::Pan => Some(Self::PanFine),
            Self::Tilt => Some(Self::TiltFine),
            Self::Red => Some(Self::RedFine),
            Self::Green => Some(Self::GreenFine),
            Self::Blue => Some(Self::BlueFine),
            Self::White => Some(Self::WhiteFine),
            Self::Zoom => Some(Self::ZoomFine),
            Self::Focus => Some(Self::FocusFine),
            _ => None,
        }
    }

    pub fn default_feature_group(&self) -> Option<DefaultFeatureGroup> {
        match self {
            Self::Intensity => Some(DefaultFeatureGroup::Intensity),

            Self::Red
            | Self::Green
            | Self::Blue
            | Self::White
            | Self::ColorTemp
            | Self::ColorTint
            | Self::ColorMacro
            | Self::ColorMacroCrossfade => Some(DefaultFeatureGroup::Color),

            Self::Focus | Self::Zoom => Some(DefaultFeatureGroup::Focus),

            Self::Shutter => Some(DefaultFeatureGroup::Beam),

            Self::Pan | Self::Tilt | Self::PanTiltSpeed => Some(DefaultFeatureGroup::Position),

            _ => None,
        }
    }
}
