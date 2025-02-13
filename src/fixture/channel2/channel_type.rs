use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::fixture::feature::group::DefaultFeatureGroup;

#[derive(
    Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Copy, Clone, EguiProbe, Default, EnumIter,
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
    pub fn get_default_feature_group(&self) -> Option<DefaultFeatureGroup> {
        match self {
            Self::Unused => None,
            Self::Intensity | Self::IntensityFine => Some(DefaultFeatureGroup::Intensity),
            Self::Pan | Self::PanFine | Self::Tilt | Self::TiltFine | Self::PanTiltSpeed => {
                Some(DefaultFeatureGroup::Position)
            }
            Self::Red
            | Self::RedFine
            | Self::Green
            | Self::GreenFine
            | Self::Blue
            | Self::BlueFine
            | Self::White
            | Self::WhiteFine
            | Self::ColorMacro
            | Self::ColorMacroCrossfade
            | Self::ColorTint
            | Self::ColorTemp => Some(DefaultFeatureGroup::Color),
            Self::Prism
            | Self::PrismRotation
            | Self::RotatingGobos
            | Self::GoboRotation
            | Self::Shutter => Some(DefaultFeatureGroup::Beam),
            Self::Focus | Self::FocusFine | Self::Zoom | Self::ZoomFine => {
                Some(DefaultFeatureGroup::Focus)
            }
            Self::ToggleFlags(_) => Some(DefaultFeatureGroup::Control),
        }
    }
}
