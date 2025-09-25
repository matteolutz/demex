use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::feature_group::FixtureChannel3FeatureGroup;

#[derive(
    Debug, Copy, Clone, strum_macros::EnumIter, PartialEq, Eq, Serialize, Deserialize, Default,
)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
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
    pub fn attributes(&self) -> &[&'static str] {
        match self {
            Self::Dimmer => &["Dimmer"],
            Self::PanTilt => &[
                "Pan",
                "Tilt",
                "PanRotate",
                "TiltRotate",
                "PositionEffect",
                "PositionEffectRate",
                "PositionEffectFade",
            ],
            Self::Rgb => &[
                "ColorAdd_R",
                "ColorAdd_G",
                "ColorAdd_B",
                "ColorAdd_C",
                "ColorAdd_M",
                "ColorAdd_Y",
                "ColorAdd_RY",
                "ColorAdd_GY",
                "ColorAdd_GC",
                "ColorAdd_BC",
                "ColorAdd_BM",
                "ColorAdd_RM",
                "ColorAdd_W",
                "ColorAdd_WW",
                "ColorAdd_CW",
                "ColorAdd_UV",
                "ColorSub_R",
                "ColorSub_G",
                "ColorSub_B",
                "ColorSub_C",
                "ColorSub_M",
                "ColorSub_Y",
                "ColorMacro(n)",
                "ColorMacro(n)Rate",
            ],
            Self::Color => &[
                "ColorEffects(n)",
                "Color(n)",
                "Color(n)WheelIndex",
                "Color(n)WheelSpin",
                "Color(n)WheelRandom",
                "Color(n)WheelAudio",
            ],
            Self::Beam => &[
                "StrobeDuration",
                "StrobeRate",
                "StrobeFrequency",
                "StrobeModeShutter",
                "StrobeModeStrobe",
                "StrobeModePulse",
                "StrobeModePulseOpen",
                "StrobeModePulseClose",
                "StrobeModeRandom",
                "StrobeModeRandomPulse",
                "StrobeModeRandomPulseOpen",
                "StrobeModeRandomPulseClose",
                "StrobeModeEffect",
                "Shutter(n)",
                "Shutter(n)Strobe",
                "Shutter(n)StrobePulse",
                "Shutter(n)StrobePulseClose",
                "Shutter(n)StrobePulseOpen",
                "Shutter(n)StrobeRandom",
                "Shutter(n)StrobeRandomPulse",
                "Shutter(n)StrobeRandomPulseClose",
                "Shutter(n)StrobeRandomPulseOpen",
                "Shutter(n)StrobeEffect",
                "Iris",
                "IrisStrobe",
                "IrisStrobeRandom",
                "IrisPulseClose",
                "IrisPulseOpen",
                "IrisRandomPulseClose",
                "IrisRandomPulseOpen",
                "Frost(n)",
                "Frost(n)PulseOpen",
                "Frost(n)PulseClose",
                "Frost(n)Ramp",
                "Prism(n)",
                "Prism(n)SelectSpin",
                "Prism(n)Macro",
                "Prism(n)Pos",
                "Prism(n)PosRotate",
                "Effects(n)",
                "Effects(n)Rate",
                "Effects(n)Fade",
                "Effects(n)Adjust(m)",
                "Effects(n)Pos",
                "Effects(n)PosRotate",
                "EffectsSync",
                "BeamShaper",
                "BeamShaperMacro",
                "BeamShaperPos",
                "BeamShaperPosRotate",
            ],
            Self::Focus => &[
                "Zoom",
                "ZoomModeSpot",
                "ZoomModeBeam",
                "DigitalZoom",
                "Focus(n)",
                "Focus(n)Adjust",
                "Focus(n)Distance",
            ],
            Self::Control => &[
                "Control(n)",
                "DimmerMode",
                "DimmerCurve",
                "BlackoutMode",
                "LEDFrequency",
                "LEDZoneMode",
                "PixelMode",
                "PanMode",
                "TiltMode",
                "PanTiltMode",
                "PositionModes",
                "Gobo(n)WheelMode",
                "GoboWheelShortcutMode",
                "AnimationWheel(n)Mode",
                "AnimationWheelShortcutMode",
                "Color(n)Mode",
                "ColorWheelShortcutMode",
                "CyanMode",
                "MagentaMode",
                "YellowMode",
                "ColorMixMode",
                "ChromaticMode",
                "ColorCalibrationMode",
                "ColorConsistency",
                "ColorControl",
                "ColorModelMode",
                "ColorSettingsReset",
                "ColorUniformity",
                "CRIMode",
                "CustomColor",
                "UVStability",
                "WaveLengthCorrection",
                "WhiteCount",
                "StrobeMode",
                "ZoomMode",
                "FocusMode",
                "IrisMode",
                "FanMode",
                "FollowSpotMode",
                "BeamEffectIndexRotateMode",
                "IntensityMSpeed",
                "PositionMSpeed",
                "ColorMixMSpeed",
                "ColorWheelSelectMSpeed",
                "GoboWheel(n)MSpeed",
                "IrisMSpeed",
                "Prism(n)MSpeed",
                "FocusMSpeed",
                "Frost(n)MSpeed",
                "ZoomMSpeed",
                "FrameMSpeed",
                "GlobalMSpeed",
                "ReflectorAdjust",
                "FixtureGlobalReset",
                "DimmerReset",
                "ShutterReset",
                "BeamReset",
                "ColorMixReset",
                "ColorWheelReset",
                "FocusReset",
                "FrameReset",
                "GoboWheelReset",
                "IntensityReset",
                "IrisReset",
                "PositionReset",
                "PanReset",
                "TiltReset",
                "ZoomReset",
                "CTBReset",
                "CTOReset",
                "CTCReset",
                "AnimationSystemReset",
                "FixtureCalibrationReset",
                "Function",
                "LampControl",
                "DisplayIntensity",
                "DMXInput",
                "NoFeature",
                "Dummy",
                "Blower(n)",
                "Fan(n)",
                "Fog(n)",
                "Haze(n)",
                "LampPowerMode",
                "Fans",
            ],
            Self::Gobo => &["Gobo(n)"],
            _ => &[],
        }
    }

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
