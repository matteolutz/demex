use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::feature_group::FixtureChannel3FeatureGroup;

#[derive(
    Debug, Copy, Clone, strum_macros::EnumIter, PartialEq, Eq, Serialize, Deserialize, Default,
)]
pub enum FixtureChannel3FeatureType {
    #[default]
    Dimmer,

    PanTilt,
    Xyz,
    Rotation,
    Scale,

    Gobo,
    Media,

    Color,
    Rgb,
    Hsb,
    Cie,
    Indirect,
    ColorCorrection,
    HsbcShift,
    ColorKey,

    Beam,

    Focus,

    Control,

    Shapers,

    Video,
}

impl FromStr for FixtureChannel3FeatureType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Dimmer" => Ok(Self::Dimmer),
            "PanTilt" => Ok(Self::PanTilt),
            "XYZ" => Ok(Self::Xyz),
            "Rotation" => Ok(Self::Rotation),
            "Scale" => Ok(Self::Scale),
            "Gobo" => Ok(Self::Gobo),
            "Media" => Ok(Self::Media),
            "Color" => Ok(Self::Color),
            "RGB" => Ok(Self::Rgb),
            "HSB" => Ok(Self::Hsb),
            "CIE" => Ok(Self::Cie),
            "Indirect" => Ok(Self::Indirect),
            "ColorCorrection" => Ok(Self::ColorCorrection),
            "HSBC_Shift" => Ok(Self::HsbcShift),
            "ColorKey" => Ok(Self::ColorKey),
            "Beam" => Ok(Self::Beam),
            "Focus" => Ok(Self::Focus),
            "Control" => Ok(Self::Control),
            "Shapers" => Ok(Self::Shapers),
            "Video" => Ok(Self::Video),
            _ => Err(()),
        }
    }
}

impl FixtureChannel3FeatureType {
    pub fn feature_group(&self) -> FixtureChannel3FeatureGroup {
        match self {
            Self::Dimmer => FixtureChannel3FeatureGroup::Dimmer,

            Self::PanTilt | Self::Xyz | Self::Rotation | Self::Scale => {
                FixtureChannel3FeatureGroup::Position
            }

            Self::Gobo | Self::Media => FixtureChannel3FeatureGroup::Gobo,

            Self::Color
            | Self::Rgb
            | Self::Hsb
            | Self::Cie
            | Self::Indirect
            | Self::ColorCorrection
            | Self::HsbcShift
            | Self::ColorKey => FixtureChannel3FeatureGroup::Color,

            Self::Beam => FixtureChannel3FeatureGroup::Beam,

            Self::Focus => FixtureChannel3FeatureGroup::Focus,

            Self::Control => FixtureChannel3FeatureGroup::Control,

            Self::Shapers => FixtureChannel3FeatureGroup::Shapers,

            Self::Video => FixtureChannel3FeatureGroup::Video,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Dimmer => "Dimmer",
            Self::PanTilt => "PanTilt",
            Self::Xyz => "XYZ",
            Self::Rotation => "Rotation",
            Self::Scale => "Scale",
            Self::Gobo => "Gobo",
            Self::Media => "Media",
            Self::Color => "Color",
            Self::Rgb => "RGB",
            Self::Hsb => "HSB",
            Self::Cie => "CIE",
            Self::Indirect => "Indirect",
            Self::ColorCorrection => "ColorCorrection",
            Self::HsbcShift => "HSBC_Shift",
            Self::ColorKey => "ColorKey",
            Self::Beam => "Beam",
            Self::Focus => "Focus",
            Self::Control => "Control",
            Self::Shapers => "Shapers",
            Self::Video => "Video",
        }
    }
}

impl std::fmt::Display for FixtureChannel3FeatureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
