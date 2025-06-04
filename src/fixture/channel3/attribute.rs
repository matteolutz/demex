use super::feature::feature_type::FixtureChannel3FeatureType;

lazy_static::lazy_static! {
    pub static ref ATTRIBUTE_WILDCARD_REGEX: regex::Regex = regex::Regex::new(r"(\d+)").unwrap();
}

// TODO: implement all
#[derive(Debug, Copy, Clone, strum_macros::EnumIter)]
pub enum FixtureChannel3Attribute {
    Dimmer,

    XzyX,
    XzyY,
    XzyZ,

    RotX,
    RotY,
    RotZ,

    ScaleX,
    ScaleY,
    ScaleZ,
    ScaleXyz,

    Gobo,
    GoboSelectSpin,
    GoboSelectShake,
    GoboSelectEffects,
    GoboWheelIndex,
    GoboWheelSpin,
    GoboWheelShake,
    GoboWheelRandom,
    GoboWheelAudio,
    GoboPos,
    GoboPosRotate,
    GoboPosShake,

    Pan,
    Tilt,
    PanRotate,
    TiltRotate,
    PositionEffect,
    PositionEffectRate,
    PositionEffectFade,

    ColorAddR,
    ColorAddG,
    ColorAddB,
}

impl FixtureChannel3Attribute {
    pub fn feature(&self) -> FixtureChannel3FeatureType {
        match self {
            Self::Dimmer => FixtureChannel3FeatureType::Dimmer,

            Self::Pan
            | Self::Tilt
            | Self::PanRotate
            | Self::TiltRotate
            | Self::PositionEffect
            | Self::PositionEffectRate
            | Self::PositionEffectFade => FixtureChannel3FeatureType::PanTilt,

            Self::XzyX | Self::XzyY | Self::XzyZ => FixtureChannel3FeatureType::Xyz,

            Self::RotX | Self::RotY | Self::RotZ => FixtureChannel3FeatureType::Rotation,

            Self::ScaleX | Self::ScaleY | Self::ScaleZ | Self::ScaleXyz => {
                FixtureChannel3FeatureType::Scale
            }

            Self::Gobo
            | Self::GoboSelectSpin
            | Self::GoboSelectShake
            | Self::GoboSelectEffects
            | Self::GoboWheelIndex
            | Self::GoboWheelSpin
            | Self::GoboWheelShake
            | Self::GoboWheelRandom
            | Self::GoboWheelAudio
            | Self::GoboPos
            | Self::GoboPosRotate
            | Self::GoboPosShake => FixtureChannel3FeatureType::Gobo,

            Self::ColorAddR | Self::ColorAddG | Self::ColorAddB => FixtureChannel3FeatureType::Rgb,
        }
    }
}

impl FixtureChannel3Attribute {
    pub fn attribute_matches(fixture_attribute_name: &str, attribute_name: &str) -> bool {
        let fixture_attribute_name =
            ATTRIBUTE_WILDCARD_REGEX.replace(fixture_attribute_name, "(n)");
        let fixture_attribute_name =
            ATTRIBUTE_WILDCARD_REGEX.replace(&fixture_attribute_name, "(m)");

        fixture_attribute_name == attribute_name
    }
}

impl std::fmt::Display for FixtureChannel3Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dimmer => write!(f, "Dimmer"),

            Self::Pan => write!(f, "Pan"),
            Self::Tilt => write!(f, "Tilt"),
            Self::PanRotate => write!(f, "PanRotate"),
            Self::TiltRotate => write!(f, "TiltRotate"),
            Self::PositionEffect => write!(f, "PositionEffect"),
            Self::PositionEffectRate => write!(f, "PositionEffectRate"),
            Self::PositionEffectFade => write!(f, "PositionEffectFade"),

            Self::XzyX => write!(f, "XYZ_X"),
            Self::XzyY => write!(f, "XYZ_Y"),
            Self::XzyZ => write!(f, "XYZ_Z"),

            Self::RotX => write!(f, "Rot_X"),
            Self::RotY => write!(f, "Rot_Y"),
            Self::RotZ => write!(f, "Rot_Z"),

            Self::ScaleX => write!(f, "Scale_X"),
            Self::ScaleY => write!(f, "Scale_Y"),
            Self::ScaleZ => write!(f, "Scale_Z"),
            Self::ScaleXyz => write!(f, "Scale_XYZ"),

            Self::Gobo => write!(f, "Gobo(n)"),
            Self::GoboSelectSpin => write!(f, "Gobo(n)SelectSpin"),
            Self::GoboSelectShake => write!(f, "Gobo(n)SelectShake"),
            Self::GoboSelectEffects => write!(f, "Gobo(n)SelectEffects"),
            Self::GoboWheelIndex => write!(f, "Gobo(n)WheelIndex"),
            Self::GoboWheelSpin => write!(f, "Gobo(n)WheelSpin"),
            Self::GoboWheelShake => write!(f, "Gobo(n)WheelShake"),
            Self::GoboWheelRandom => write!(f, "Gobo(n)WheelRandom"),
            Self::GoboWheelAudio => write!(f, "Gobo(n)WheelAudio"),
            Self::GoboPos => write!(f, "Gobo(n)Pos"),
            Self::GoboPosRotate => write!(f, "Gobo(n)PosRotate"),
            Self::GoboPosShake => write!(f, "Gobo(n)PosShake"),

            Self::ColorAddR => write!(f, "ColorAdd_R"),
            Self::ColorAddG => write!(f, "ColorAdd_G"),
            Self::ColorAddB => write!(f, "ColorAdd_B"),
        }
    }
}
