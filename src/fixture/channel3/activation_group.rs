#[derive(Debug, Copy, Clone)]
pub enum FixtureChannel3ActivationGroup {
    PanTilt,
    Xyz,
    RotXyz,
    ScaleXyz,
    ColorRgb,
    ColorHsb,
    ColorCie,
    ColorIndirect,
    Gobo(usize),
    GoboPos(usize),
    AnimationWheel(usize),
    AnimationWheelPos(usize),
    AnimationSystem(usize),
    AnimationSystemPos(usize),
    Prism,
    BeamShaper,
    Shaper,
}

impl FixtureChannel3ActivationGroup {
    pub fn attributes(&self) -> &[&str] {
        match self {
            _ => &[],
        }
    }
}

impl ToString for FixtureChannel3ActivationGroup {
    fn to_string(&self) -> String {
        match self {
            Self::PanTilt => "PanTilt".to_string(),
            Self::Xyz => "XYZ".to_string(),
            Self::RotXyz => "Rot_XYZ".to_string(),
            Self::ScaleXyz => "Scale_XYZ".to_string(),
            Self::ColorRgb => "ColorRGB".to_string(),
            Self::ColorHsb => "ColorHSB".to_string(),
            Self::ColorCie => "ColorCIE".to_string(),
            Self::ColorIndirect => "ColorIndirect".to_string(),
            Self::Gobo(n) => format!("Gobo{}", n),
            Self::GoboPos(n) => format!("Gobo{}Pos", n),
            Self::AnimationWheel(n) => format!("AnimationWheel{}", n),
            Self::AnimationWheelPos(n) => format!("AnimationWheel{}Pos", n),
            Self::AnimationSystem(n) => format!("AnimationSystem{}", n),
            Self::AnimationSystemPos(n) => format!("AnimationSystem{}Pos", n),
            Self::Prism => "Prism".to_string(),
            Self::BeamShaper => "BeamShaper".to_string(),
            Self::Shaper => "Shaper".to_string(),
        }
    }
}
