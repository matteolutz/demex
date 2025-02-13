use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Copy, Clone, EguiProbe, Default)]
pub enum FixtureChannelType {
    #[default]
    Intensity,

    IntensityFine,
    Pan,
    PanFine,
    Tilt,
    TiltFine,
    Red,
    RedFine,
    Green,
    GreenFine,
    Blue,
    BlueFine,
    ToggleFlags(usize),
}
