/*
 * This file is part of zeevonk (https://github.com/BaukeWestendorp/zeevonk).
 *
 * Original work:
 *   Copyright (C) 2025 Bauke Westendorp
 *
 * Modifications:
 *   Copyright (C) 2025 Matteo Lutz
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 */

//! Attribute helpers and types used by the GDCS.
//!
//! This module contains types and utilities related to fixture attributes
//! (e.g. pan, tilt, color) that are used when setting and resolving channel
//! function values.

use std::fmt;
use std::str::FromStr;
use std::sync::Mutex;

use crate::channel3::feature::feature_type::FixtureChannel3FeatureType;

lazy_static::lazy_static! {
    static ref CUSTOM_NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

/// A GDTF attribute.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixtureChannel3Attribute {
    /// Controls the intensity of a fixture.
    Dimmer,

    /// Controls the fixture’s sideward movement (horizontal axis).
    Pan,
    /// Controls the fixture’s upward and the downward movement (vertical axis).
    Tilt,
    /// Controls the speed of the fixture’s continuous pan movement (horizontal
    /// axis).
    PanRotate,
    /// Controls the speed of the fixture’s continuous tilt movement (vertical
    /// axis).
    TiltRotate,
    /// Selects the predefined position effects that are built into the fixture.
    PositionEffect,
    /// Controls the speed of the predefined position effects that are built
    /// into the fixture.
    PositionEffectRate,
    /// Snaps or smooth fades with timing in running predefined position
    /// effects.
    PositionEffectFade,
    /// Defines a fixture’s x-coordinate within an XYZ coordinate system.
    XyzX,
    /// Defines a fixture’s y-coordinate within an XYZ coordinate system.
    XyzY,
    /// Defines a fixture‘s z-coordinate within an XYZ coordinate system.
    XyzZ,
    /// Defines rotation around X axis.
    RotX,
    /// Defines rotation around Y axis.
    RotY,
    /// Defines rotation around Z axis.
    RotZ,
    /// Scaling on X axis.
    ScaleX,
    /// Scaling on Y axis.
    ScaleY,
    /// Scaling on Y axis.
    ScaleZ,
    /// Unified scaling on all axis.
    ScaleXYZ,

    /// The fixture’s gobo wheel (n). This is the main attribute of gobo wheel’s
    /// (n) wheel control. Selects gobos in gobo wheel (n). A different channel
    /// function sets the angle of the indexed position in the selected gobo or
    /// the angular speed of its continuous rotation.
    Gobo(u8),
    /// Selects gobos whose rotation is continuous in gobo wheel (n) and
    /// controls the angular speed of the gobo’s spin within the same channel
    /// function.
    GoboSelectSpin(u8),
    /// Selects gobos which shake in gobo wheel (n) and controls the frequency
    /// of the gobo’s shake within the same channel function.
    GoboSelectShake(u8),
    /// Selects gobos which run effects in gobo wheel (n).
    GoboSelectEffects(u8),
    /// Controls angle of indexed rotation of gobo wheel (n).
    GoboWheelIndex(u8),
    /// Controls the speed and direction of continuous rotation of gobo wheel
    /// (n).
    GoboWheelSpin(u8),
    /// Controls frequency of the shake of gobo wheel (n).
    GoboWheelShake(u8),
    /// Controls speed of gobo wheel´s (n) random gobo slot selection.
    GoboWheelRandom(u8),
    /// Controls audio-controlled functionality of gobo wheel (n).
    GoboWheelAudio(u8),
    /// Controls angle of indexed rotation of gobos in gobo wheel (n). This is
    /// the main attribute of gobo wheel’s (n) wheel slot control.
    GoboPos(u8),
    /// Controls the speed and direction of continuous rotation of gobos in gobo
    /// wheel (n).
    GoboPosRotate(u8),
    /// Controls frequency of the shake of gobos in gobo wheel (n).
    GoboPosShake(u8),
    /// This is the main attribute of the animation wheel’s (n) wheel control.
    /// Selects slots in the animation wheel. A different channel function sets
    /// the angle of the indexed position in the selected slot or the angular
    /// speed of its continuous rotation. Is used for animation effects with
    /// multiple slots.
    AnimationWheel(u8),
    /// Controls audio-controlled functionality of animation wheel (n).
    AnimationWheelAudio(u8),
    /// Selects predefined effects in animation wheel (n).
    AnimationWheelMacro(u8),
    /// Controls frequency of animation wheel (n) random slot selection.
    AnimationWheelRandom(u8),
    /// Selects slots which run effects in animation wheel (n).
    AnimationWheelSelectEffects(u8),
    /// Selects slots which shake in animation wheel and controls the frequency
    /// of the slots shake within the same channel function.
    AnimationWheelSelectShake(u8),
    /// Selects slots whose rotation is continuous in animation wheel and
    /// controls the angular speed of the slot spin within the same channel
    /// function
    AnimationWheelSelectSpin(u8),
    /// Controls angle of indexed rotation of slots in animation wheel. This is
    /// the main attribute of animation wheel (n) wheel slot control.
    AnimationWheelPos(u8),
    /// Controls the speed and direction of continuous rotation of slots in
    /// animation wheel (n).
    AnimationWheelPosRotate(u8),
    /// Controls frequency of the shake of slots in animation wheel (n).
    AnimationWheelPosShake(u8),
    /// This is the main attribute of the animation system insertion control.
    /// Controls the insertion of the fixture’s animation system in the light
    /// output. Is used for animation effects where a disk is inserted into the
    /// light output.
    AnimationSystem(u8),
    /// Sets frequency of animation system (n) insertion ramp.
    AnimationSystemRamp(u8),
    /// Sets frequency of animation system (n) insertion shake.
    AnimationSystemShake(u8),
    /// Controls audio-controlled functionality of animation system (n)
    /// insertion.
    AnimationSystemAudio(u8),
    /// Controls frequency of animation system (n) random insertion.
    AnimationSystemRandom(u8),
    /// This is the main attribute of the animation system spinning control.
    /// Controls angle of indexed rotation of animation system (n) disk.
    AnimationSystemPos(u8),
    /// Controls the speed and direction of continuous rotation of animation
    /// system (n) disk.
    AnimationSystemPosRotate(u8),
    /// Controls frequency of the shake of animation system (n) disk.
    AnimationSystemPosShake(u8),
    /// Controls random speed of animation system (n) disk.
    AnimationSystemPosRandom(u8),
    /// Controls audio-controlled functionality of animation system (n) disk.
    AnimationSystemPosAudio(u8),
    /// Selects predefined effects in animation system (n).
    AnimationSystemMacro(u8),
    /// Selects folder that contains media content.
    MediaFolder(u8),
    /// Selects file with media content.
    MediaContent(u8),
    /// Selects folder that contains 3D model content. For example 3D meshes for
    /// mapping.
    ModelFolder(u8),
    /// Selects file with 3D model content.
    ModelContent(u8),
    /// Defines media playback mode.
    PlayMode,
    /// Defines starting point of media content playback.
    PlayBegin,
    /// Defines end point of media content playback.
    PlayEnd,
    /// Adjusts playback speed of media content.
    PlaySpeed,

    /// Selects predefined color effects built into the fixture.
    ColorEffects(u8),
    /// The fixture’s color wheel (n). Selects colors in color wheel (n). This
    /// is the main attribute of color wheel’s (n) wheel control.
    Color(u8),
    /// Controls angle of indexed rotation of color wheel (n)
    ColorWheelIndex(u8),
    /// Controls the speed and direction of continuous rotation of color wheel
    /// (n).
    ColorWheelSpin(u8),
    /// Controls frequency of color wheel´s (n) random color slot selection.
    ColorWheelRandom(u8),
    /// Controls audio-controlled functionality of color wheel (n).
    ColorWheelAudio(u8),
    /// Controls the intensity of the fixture’s red emitters for direct additive
    /// color mixing.
    ColorAddR,
    /// Controls the intensity of the fixture’s green emitters for direct
    /// additive color mixing
    ColorAddG,
    /// Controls the intensity of the fixture’s blue emitters for direct
    /// additive color mixing.
    ColorAddB,
    /// Controls the intensity of the fixture’s cyan emitters for direct
    /// additive color mixing.
    ColorAddC,
    /// Controls the intensity of the fixture’s magenta emitters for direct
    /// additive color mixing.
    ColorAddM,
    /// Controls the intensity of the fixture’s yellow emitters for direct
    /// additive color mixing.
    ColorAddY,
    /// Controls the intensity of the fixture’s amber emitters for direct
    /// additive color mixing.
    ColorAddRY,
    /// Controls the intensity of the fixture’s lime emitters for direct
    /// additive color mixing.
    ColorAddGY,
    /// Controls the intensity of the fixture’s blue-green emitters for direct
    /// additive color mixing.
    ColorAddGC,
    /// Controls the intensity of the fixture’s light-blue emitters for direct
    /// additive color mixing.
    ColorAddBC,
    /// Controls the intensity of the fixture’s purple emitters for direct
    /// additive color mixing.
    ColorAddBM,
    /// Controls the intensity of the fixture’s pink emitters for direct
    /// additive color mixing.
    ColorAddRM,
    /// Controls the intensity of the fixture’s white emitters for direct
    /// additive color mixing.
    ColorAddW,
    /// Controls the intensity of the fixture’s warm white emitters for direct
    /// additive color mixing.
    ColorAddWW,
    /// Controls the intensity of the fixture’s cool white emitters for direct
    /// additive color mixing.
    ColorAddCW,
    /// Controls the intensity of the fixture’s UV emitters for direct additive
    /// color mixing.
    ColorAddUV,
    /// Controls the insertion of the fixture’s red filter flag for direct
    /// subtractive color mixing.
    ColorSubR,
    /// Controls the insertion of the fixture’s green filter flag for direct
    /// subtractive color mixing.
    ColorSubG,
    /// Controls the insertion of the fixture’s blue filter flag for direct
    /// subtractive color mixing.
    ColorSubB,
    /// Controls the insertion of the fixture’s cyan filter flag for direct
    /// subtractive color mixing.
    ColorSubC,
    /// Controls the insertion of the fixture’s magenta filter flag for direct
    /// subtractive color mixing.
    ColorSubM,
    /// Controls the insertion of the fixture’s yellow filter flag for direct
    /// subtractive color mixing.
    ColorSubY,
    /// Selects predefined colors that are programed in the fixture’s firmware.
    ColorMacro(u8),
    /// Controls the time between Color Macro steps.
    ColorMacroRate(u8),
    /// Controls the fixture’s “Correct to orange” wheel or mixing system.
    Cto,
    /// Controls the fixture’s “Correct to color” wheel or mixing system.
    Ctc,
    /// Controls the fixture’s “Correct to blue” wheel or mixing system.
    Ctb,
    /// Controls the fixture’s “Correct green to magenta” wheel or mixing
    /// system.
    Tint,
    /// Controls the fixture’s color attribute regarding the hue.
    HsbHue,
    /// Controls the fixture’s color attribute regarding the saturation.
    HsbSaturation,
    /// Controls the fixture’s color attribute regarding the brightness.
    HsbBrightness,
    /// Controls the fixture’s color attribute regarding the quality.
    HsbQuality,
    /// Controls the fixture’s CIE 1931 color attribute regarding the
    /// chromaticity x.
    CieX,
    /// Controls the fixture’s CIE 1931 color attribute regarding the
    /// chromaticity y.
    CieY,
    /// Controls the fixture’s CIE 1931 color attribute regarding the brightness
    /// (Y).
    CieBrightness,
    /// Controls the fixture’s red attribute for indirect RGB color mixing.
    ColorRgbRed,
    /// Controls the fixture’s green attribute for indirect RGB color mixing.
    ColorRgbGreen,
    /// Controls the fixture’s blue attribute for indirect RGB color mixing.
    ColorRgbBlue,
    /// Controls the fixture’s cyan attribute for indirect CMY color mixing.
    ColorRgbCyan,
    /// Controls the fixture’s magenta attribute for indirect CMY color mixing.
    ColorRgbMagenta,
    /// Controls the fixture’s yellow attribute for indirect CMY color mixing.
    ColorRgbYellow,
    /// Controls the fixture’s quality attribute for indirect color mixing.
    ColorRgbQuality,
    /// Adjusts color boost red of content.
    VideoBoostR,
    /// Adjusts color boost green of content.
    VideoBoostG,
    /// Adjusts color boost blue of content.
    VideoBoostB,
    /// Adjusts color hue shift of content.
    VideoHueShift,
    /// Adjusts saturation of content.
    VideoSaturation,
    /// Adjusts brightness of content.
    VideoBrightness,
    /// Adjusts contrast of content.
    VideoContrast,
    /// Adjusts red color for color keying.
    VideoKeyColorR,
    /// Adjusts green color for color keying.
    VideoKeyColorG,
    /// Adjusts blue color for color keying.
    VideoKeyColorB,
    /// Adjusts intensity of color keying.
    VideoKeyIntensity,
    /// Adjusts tolerance of color keying.
    VideoKeyTolerance,

    /// Controls the length of a strobe flash.
    StrobeDuration,
    /// Controls the time between strobe flashes.
    StrobeRate,
    /// Controls the frequency of strobe flashes.
    StrobeFrequency,
    /// Strobe mode shutter. Use this attribute together with StrobeFrequency to
    /// define the type of the shutter / strobe.
    StrobeModeShutter,
    /// Strobe mode strobe. Use this attribute together with StrobeFrequency to
    /// define the type of the shutter / strobe.
    StrobeModeStrobe,
    /// Strobe mode pulse. Use this attribute together with StrobeFrequency to
    /// define the type of the shutter / strobe.
    StrobeModePulse,
    /// Strobe mode opening pulse. Use this attribute together with
    /// StrobeFrequency to define the type of the shutter / strobe.
    StrobeModePulseOpen,
    /// Strobe mode closing pulse. Use this attribute together with
    /// StrobeFrequency to define the type of the shutter / strobe.
    StrobeModePulseClose,
    /// Strobe mode random strobe. Use this attribute together with
    /// StrobeFrequency to define the type of the shutter / strobe.
    StrobeModeRandom,
    /// Strobe mode random pulse. Use this attribute together with
    /// StrobeFrequency to define the type of the shutter / strobe.
    StrobeModeRandomPulse,
    /// Strobe mode random opening pulse. Use this attribute together with
    /// StrobeFrequency to define the type of the shutter / strobe.
    StrobeModeRandomPulseOpen,
    /// Strobe mode random closing pulse. Use this attribute together with
    /// StrobeFrequency to define the type of the shutter / strobe.
    StrobeModeRandomPulseClose,
    /// Strobe mode random shutter effect feature. Use this attribute together
    /// with StrobeFrequency to define the type of the shutter / strobe.
    StrobeModeEffect,
    /// Controls the fixture´s mechanical or electronical shutter feature.
    Shutter(u8),
    /// Controls the frequency of the fixture´s mechanical or electronical
    /// strobe shutter feature.
    ShutterStrobe(u8),
    /// Controls the frequency of the fixture´s mechanical or electronical pulse
    /// shutter feature.
    ShutterStrobePulse(u8),
    /// Controls the frequency of the fixture´s mechanical or electronical
    /// closing pulse shutter feature. The pulse is described by a ramp
    /// function.
    ShutterStrobePulseClose(u8),
    /// Controls the frequency of the fixture´s mechanical or electronical
    /// opening pulse shutter feature. The pulse is described by a ramp
    /// function.
    ShutterStrobePulseOpen(u8),
    /// Controls the frequency of the fixture´s mechanical or electronical
    /// random strobe shutter feature.
    ShutterStrobeRandom(u8),
    /// Controls the frequency of the fixture´s mechanical or electronical
    /// random pulse shutter feature.
    ShutterStrobeRandomPulse(u8),
    /// Controls the frequency of the fixture´s mechanical or electronical
    /// random closing pulse shutter feature. The pulse is described by a ramp
    /// function.
    ShutterStrobeRandomPulseClose(u8),
    /// Controls the frequency of the fixture´s mechanical or electronical
    /// random opening pulse shutter feature. The pulse is described by a ramp
    /// function.
    ShutterStrobeRandomPulseOpen(u8),
    /// Controls the frequency of the fixture´s mechanical or electronical
    /// shutter effect feature.
    ShutterStrobeEffect(u8),
    /// Controls the diameter of the fixture’s beam.
    Iris,
    /// Sets frequency of the iris’s strobe feature.
    IrisStrobe,
    /// Sets frequency of the iris’s random movement.
    IrisStrobeRandom,
    /// Sets frequency of iris’s closing pulse.
    IrisPulseClose,
    /// Sets frequency of iris’s opening pulse.
    IrisPulseOpen,
    /// Sets frequency of iris’s random closing pulse.
    IrisRandomPulseClose,
    /// Sets frequency of iris’s random opening pulse.
    IrisRandomPulseOpen,
    /// The ability to soften the fixture’s spot light with a frosted lens.
    Frost(u8),
    /// Sets frequency of frost’s opening pulse
    FrostPulseOpen(u8),
    /// Sets frequency of frost’s closing pulse.
    FrostPulseClose(u8),
    /// Sets frequency of frost’s ramp.
    FrostRamp(u8),
    /// The fixture's prism wheel (n). Selects prisms in prism wheel (n). A
    /// different channel function sets the angle of the indexed position in the
    /// selected prism or the angular speed of its continuous rotation. This is
    /// the main attribute of prism wheel's (n) wheel control.
    Prism(u8),
    /// Selects prisms whose rotation is continuous in prism wheel (n) and
    /// controls the angular speed of the prism's spin within the same channel
    /// function.
    PrismSelectSpin(u8),
    /// Macro functions of prism wheel (n).
    PrismMacro(u8),
    /// Controls angle of indexed rotation of prisms in prism wheel (n). This is
    /// the main attribute of prism wheel's 1 wheel slot control.
    PrismPos(u8),
    /// Controls the speed and direction of continuous rotation of prisms in
    /// prism wheel (n).
    PrismPosRotate(u8),

    /// Generically predefined macros and effects of a fixture.
    Effects(u8),
    /// Frequency of running effects.
    EffectsRate(u8),
    /// Snapping or smooth look of running effects.
    EffectsFade(u8),
    /// Controls parameter (m) of effect (n).
    EffectsAdjust(u8, u8),
    /// Controls angle of indexed rotation of slot/effect in effect wheel/macro
    /// (n). This is the main attribute of effect wheel/macro (n) slot/effect
    /// control.
    EffectsPos(u8),
    /// Controls speed and direction of slot/effect in effect wheel (n).
    EffectsPosRotate(u8),
    /// Sets offset between running effects and effects 2.
    EffectsSync,
    /// Activates fixture's beam shaper.
    BeamShaper,
    /// Predefined presets for fixture's beam shaper positions.
    BeamShaperMacro,
    /// Indexing of fixture's beam shaper.
    BeamShaperPos,
    /// Continuous rotation of fixture's beam shaper.
    BeamShaperPosRotate,
    /// Controls the spread of the fixture's beam/spot.
    Zoom,
    /// Selects spot mode of zoom.
    ZoomModeSpot,
    /// Selects beam mode of zoom.
    ZoomModeBeam,
    /// Controls the image size within the defined projection. Used on digital
    /// projection based devices.
    DigitalZoom,
    /// Controls the sharpness of the fixture's spot light. Can blur or sharpen
    /// the edge of the spot.
    Focus(u8),
    /// Autofocuses functionality using presets.
    FocusAdjust(u8),
    /// Autofocuses functionality using distance.
    FocusDistance(u8),

    /// Controls the channel of a fixture.
    Control(u8),
    /// Selects different modes of intensity.
    DimmerMode,
    /// Selects different dimmer curves of the fixture.
    DimmerCurve,
    /// Close the light output under certain conditions (movement correction,
    /// gobo movement, etc.).
    BlackoutMode,
    /// Controls LED frequency.
    LedFrequency,
    /// Changes zones of LEDs.
    LedZoneMode,
    /// Controls behavior of LED pixels.
    PixelMode,
    /// Selects fixture's pan mode. Selects between a limited pan range (e.g.
    /// -270 to 270) or a continuous pan range.
    PanMode,
    /// Selects fixture's pan mode. Selects between a limited tilt range (e.g.
    /// -130 to 130) or a continuous tilt range.
    TiltMode,
    /// Selects fixture's pan/tilt mode. Selects between a limited pan/tilt
    /// range or a continuous pan/tilt range.
    PanTiltMode,
    /// Selects the fixture's position mode.
    PositionModes,
    /// Changes control between selecting, indexing, and rotating the gobos of
    /// gobo wheel (n).
    GoboWheelMode(u8),
    /// Defines whether the gobo wheel takes the shortest distance between two
    /// positions.
    GoboWheelShortcutMode,
    /// Changes control between selecting, indexing, and rotating the slots of
    /// animation wheel (n).
    AnimationWheelMode(u8),
    /// Defines whether the animation wheel takes the shortest distance between
    /// two positions.
    AnimationWheelShortcutMode,
    /// Changes control between selecting, continuous selection, half selection,
    /// random selection, color spinning, etc. in colors of color wheel (n).
    ColorMode(u8),
    /// Defines whether the color wheel takes the shortest distance between two
    /// colors.
    ColorWheelShortcutMode,
    /// Controls how Cyan is used within the fixture's cyan CMY-mixing feature.
    CyanMode,
    /// Controls how Cyan is used within the fixture's magenta CMY-mixing.
    MagentaMode,
    /// Controls how Cyan is used within the fixture's yellow CMY-mixing
    /// feature.
    YellowMode,
    /// Changes control between selecting continuous selection, half selection,
    /// random selection, color spinning, etc. in color mixing.
    ColorMixMode,
    /// Selects chromatic behavior of the device.
    ChromaticMode,
    /// Sets calibration mode (for example on/off).
    ColorCalibrationMode,
    /// Controls consistent behavior of color.
    ColorConsistency,
    /// Controls special color related functions.
    ColorControl,
    /// Controls color model (CMY/RGB/HSV…).
    ColorModelMode,
    /// Resets settings of color control channel.
    ColorSettingsReset,
    /// Controls behavior of color uniformity.
    ColorUniformity,
    /// Controls CRI settings of output.
    CriMode,
    /// Custom color related functions (save, recall..).
    CustomColor,
    /// Settings for UV stability color behavior.
    UvStability,
    /// Settings for Wavelength correction of colors.
    WavelengthCorrection,
    /// Controls if White LED is proportionally added to RGB.
    WhiteCount,
    /// Changes strobe style - strobe, pulse, random strobe, etc. - of the
    /// shutter attribute.
    StrobeMode,
    /// Changes modes of the fixture´s zoom.
    ZoomMode,
    /// Changes modes of the fixture's focus - manual or auto- focus.
    FocusMode,
    /// Changes modes of the fixture's iris - linear, strobe, pulse.
    IrisMode,
    /// Controls fan (n) mode.
    FanMode(u8),
    /// Selects follow spot control mode.
    FollowSpotMode,
    /// Changes mode to control either index or rotation of the beam effects.
    BeamEffectIndexRotateMode,
    /// Movement speed of the fixture's intensity.
    IntensityMSpeed,
    /// Movement speed of the fixture's pan/tilt.
    PositionMSpeed,
    /// Movement speed of the fixture's ColorMix presets.
    ColorMixMSpeed,
    /// Movement speed of the fixture's color wheel.
    ColorWheelSelectMSpeed,
    /// Movement speed of the fixture's gobo wheel (n).
    GoboWheelMSpeed(u8),
    /// Movement speed of the fixture's iris.
    IrisMSpeed,
    /// Movement speed of the fixture's prism wheel (n).
    PrismMSpeed(u8),
    /// Movement speed of the fixture's focus.
    FocusMSpeed,
    /// Movement speed of the fixture's frost (n).
    FrostMSpeed(u8),
    /// Movement speed of the fixture's zoom.
    ZoomMSpeed,
    /// Movement speed of the fixture's shapers.
    FrameMSpeed,
    /// General speed of fixture's features.
    GlobalMSpeed,
    /// Movement speed of the fixture's frost.
    ReflectorAdjust,
    /// Generally resets the entire fixture.
    FixtureGlobalReset,
    /// Resets the fixture's dimmer.
    DimmerReset,
    /// Resets the fixture's shutter.
    ShutterReset,
    /// Resets the fixture's beam features.
    BeamReset,
    /// Resets the fixture's color mixing system.
    ColorMixReset,
    /// Resets the fixture's color wheel.
    ColorWheelReset,
    /// Resets the fixture's focus.
    FocusReset,
    /// Resets the fixture's shapers.
    FrameReset,
    /// Resets the fixture's gobo wheel.
    GoboWheelReset,
    /// Resets the fixture's intensity.
    IntensityReset,
    /// Resets the fixture's iris.
    IrisReset,
    /// Resets the fixture's pan/tilt.
    PositionReset,
    /// Resets the fixture's pan.
    PanReset,
    /// Resets the fixture's tilt.
    TiltReset,
    /// Resets the fixture's zoom.
    ZoomReset,
    /// Resets the fixture's CTB.
    CtbReset,
    /// Resets the fixture's CTO.
    CtoReset,
    /// Resets the fixture's CTC.
    CtcReset,
    /// Resets the fixture's animation system features.
    AnimationSystemReset,
    /// Resets the fixture's calibration.
    FixtureCalibrationReset,
    /// Generally controls features of the fixture.
    Function,
    /// Controls the fixture's lamp on/lamp off feature.
    LampControl,
    /// Adjusts intensity of display.
    DisplayIntensity,
    /// Selects DMX Input.
    DmxInput,
    /// Ranges without a functionality.
    NoFeature,
    /// Fog or hazer's blower feature.
    Blower(u8),
    /// Fog or hazer's Fan feature.
    Fan(u8),
    /// Fog or hazer's Fog feature.
    Fog(u8),
    /// Fog or hazer's Haze feature.
    Haze(u8),
    /// Controls the energy consumption of the lamp.
    LampPowerMode,
    /// Controls a fixture or device fan.
    Fans,

    /// 1 of 2 shutters that shape the top/right/bottom/left of the beam.
    BladeA(u8),
    /// 2 of 2 shutters that shape the top/right/bottom/left of the beam.
    BladeB(u8),
    /// Rotates position of blade(n).
    BladeRot(u8),
    /// Rotates position of blade assembly.
    ShaperRot,
    /// Predefined presets for shaper positions.
    ShaperMacros,
    /// Speed of predefined effects on shapers.
    ShaperMacrosSpeed,
    /// 1 of 2 soft edge blades that shape the top/right/bottom/left of the
    /// beam.
    BladeSoftA(u8),
    /// 2 of 2 soft edge blades that shape the top/right/bottom/left of the
    /// beam.
    BladeSoftB(u8),
    /// 1 of 2 corners that shape the top/right/bottom/left of the beam.
    KeyStoneA(u8),
    /// 2 of 2 corners that shape the top/right/bottom/left of the beam.
    KeyStoneB(u8),

    /// Controls video features.
    Video,
    /// Selects dedicated effects which are used for media.
    VideoEffectType(u8),
    /// Controls parameter (m) of VideoEffect(n)Type.
    VideoEffectParameter(u8, u8),
    /// Selects the video camera(n).
    VideoCamera(u8),
    /// Adjusts sound volume.
    VideoSoundVolume(u8),
    /// Defines mode of video blending.
    VideoBlendMode,
    /// Defines media input source e.g. a camera input.
    InputSource,
    /// Defines field of view.
    FieldOfView,

    /// Any other non-standard attribute.
    Custom(CustomName),
}

/// Wrapper to make sure [`Attribute`] is [`Copy`].
///
/// To get the actual name of the custom attribute, you can use [CustomName::to_string].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CustomName(usize);

impl CustomName {
    fn new(s: String) -> Self {
        let mut names = CUSTOM_NAMES.lock().unwrap();
        if let Some(ix) = names.iter().position(|name| name == &s) {
            Self(ix)
        } else {
            let ix = names.len();
            names.push(s);
            Self(ix)
        }
    }
}

impl ToString for CustomName {
    fn to_string(&self) -> String {
        CUSTOM_NAMES.lock().unwrap()[self.0].to_owned()
    }
}

impl FixtureChannel3Attribute {
    /// Increment the attribute value by one.
    /// - For attributes with a single parameter [`n`], increment [`n`] by one.
    /// - For attributes with two parameters [`n`] and [`m`], increment [`m`] by one.
    pub fn try_increment(&self) -> Option<Self> {
        match self {
            Self::Gobo(n) => Some(Self::Gobo(n + 1)),
            Self::GoboSelectSpin(n) => Some(Self::GoboSelectSpin(n + 1)),
            Self::GoboSelectShake(n) => Some(Self::GoboSelectShake(n + 1)),
            Self::GoboSelectEffects(n) => Some(Self::GoboSelectEffects(n + 1)),
            Self::GoboWheelIndex(n) => Some(Self::GoboWheelIndex(n + 1)),
            Self::GoboWheelSpin(n) => Some(Self::GoboWheelSpin(n + 1)),
            Self::GoboWheelShake(n) => Some(Self::GoboWheelShake(n + 1)),
            Self::GoboWheelRandom(n) => Some(Self::GoboWheelRandom(n + 1)),
            Self::GoboWheelAudio(n) => Some(Self::GoboWheelAudio(n + 1)),
            Self::GoboPos(n) => Some(Self::GoboPos(n + 1)),
            Self::GoboPosRotate(n) => Some(Self::GoboPosRotate(n + 1)),
            Self::GoboPosShake(n) => Some(Self::GoboPosShake(n + 1)),
            Self::AnimationWheel(n) => Some(Self::AnimationWheel(n + 1)),
            Self::AnimationWheelAudio(n) => Some(Self::AnimationWheelAudio(n + 1)),
            Self::AnimationWheelMacro(n) => Some(Self::AnimationWheelMacro(n + 1)),
            Self::AnimationWheelRandom(n) => Some(Self::AnimationWheelRandom(n + 1)),
            Self::AnimationWheelSelectEffects(n) => Some(Self::AnimationWheelSelectEffects(n + 1)),
            Self::AnimationWheelSelectShake(n) => Some(Self::AnimationWheelSelectShake(n + 1)),
            Self::AnimationWheelSelectSpin(n) => Some(Self::AnimationWheelSelectSpin(n + 1)),
            Self::AnimationWheelPos(n) => Some(Self::AnimationWheelPos(n + 1)),
            Self::AnimationWheelPosRotate(n) => Some(Self::AnimationWheelPosRotate(n + 1)),
            Self::AnimationWheelPosShake(n) => Some(Self::AnimationWheelPosShake(n + 1)),
            Self::AnimationSystem(n) => Some(Self::AnimationSystem(n + 1)),
            Self::AnimationSystemRamp(n) => Some(Self::AnimationSystemRamp(n + 1)),
            Self::AnimationSystemShake(n) => Some(Self::AnimationSystemShake(n + 1)),
            Self::AnimationSystemAudio(n) => Some(Self::AnimationSystemAudio(n + 1)),
            Self::AnimationSystemRandom(n) => Some(Self::AnimationSystemRandom(n + 1)),
            Self::AnimationSystemPos(n) => Some(Self::AnimationSystemPos(n + 1)),
            Self::AnimationSystemPosRotate(n) => Some(Self::AnimationSystemPosRotate(n + 1)),
            Self::AnimationSystemPosShake(n) => Some(Self::AnimationSystemPosShake(n + 1)),
            Self::AnimationSystemPosRandom(n) => Some(Self::AnimationSystemPosRandom(n + 1)),
            Self::AnimationSystemPosAudio(n) => Some(Self::AnimationSystemPosAudio(n + 1)),
            Self::AnimationSystemMacro(n) => Some(Self::AnimationSystemMacro(n + 1)),
            Self::MediaFolder(n) => Some(Self::MediaFolder(n + 1)),
            Self::MediaContent(n) => Some(Self::MediaContent(n + 1)),
            Self::ModelFolder(n) => Some(Self::ModelFolder(n + 1)),
            Self::ModelContent(n) => Some(Self::ModelContent(n + 1)),
            Self::ColorEffects(n) => Some(Self::ColorEffects(n + 1)),
            Self::Color(n) => Some(Self::Color(n + 1)),
            Self::ColorWheelIndex(n) => Some(Self::ColorWheelIndex(n + 1)),
            Self::ColorWheelSpin(n) => Some(Self::ColorWheelSpin(n + 1)),
            Self::ColorWheelRandom(n) => Some(Self::ColorWheelRandom(n + 1)),
            Self::ColorWheelAudio(n) => Some(Self::ColorWheelAudio(n + 1)),
            Self::ColorMacro(n) => Some(Self::ColorMacro(n + 1)),
            Self::ColorMacroRate(n) => Some(Self::ColorMacroRate(n + 1)),
            Self::Shutter(n) => Some(Self::Shutter(n + 1)),
            Self::ShutterStrobe(n) => Some(Self::ShutterStrobe(n + 1)),
            Self::ShutterStrobePulse(n) => Some(Self::ShutterStrobePulse(n + 1)),
            Self::ShutterStrobePulseClose(n) => Some(Self::ShutterStrobePulseClose(n + 1)),
            Self::ShutterStrobePulseOpen(n) => Some(Self::ShutterStrobePulseOpen(n + 1)),
            Self::ShutterStrobeRandom(n) => Some(Self::ShutterStrobeRandom(n + 1)),
            Self::ShutterStrobeRandomPulse(n) => Some(Self::ShutterStrobeRandomPulse(n + 1)),
            Self::ShutterStrobeRandomPulseClose(n) => {
                Some(Self::ShutterStrobeRandomPulseClose(n + 1))
            }
            Self::ShutterStrobeRandomPulseOpen(n) => {
                Some(Self::ShutterStrobeRandomPulseOpen(n + 1))
            }
            Self::ShutterStrobeEffect(n) => Some(Self::ShutterStrobeEffect(n + 1)),
            Self::Frost(n) => Some(Self::Frost(n + 1)),
            Self::FrostPulseOpen(n) => Some(Self::FrostPulseOpen(n + 1)),
            Self::FrostPulseClose(n) => Some(Self::FrostPulseClose(n + 1)),
            Self::FrostRamp(n) => Some(Self::FrostRamp(n + 1)),
            Self::Prism(n) => Some(Self::Prism(n + 1)),
            Self::PrismSelectSpin(n) => Some(Self::PrismSelectSpin(n + 1)),
            Self::PrismMacro(n) => Some(Self::PrismMacro(n + 1)),
            Self::PrismPos(n) => Some(Self::PrismPos(n + 1)),
            Self::PrismPosRotate(n) => Some(Self::PrismPosRotate(n + 1)),
            Self::Effects(n) => Some(Self::Effects(n + 1)),
            Self::EffectsRate(n) => Some(Self::EffectsRate(n + 1)),
            Self::EffectsFade(n) => Some(Self::EffectsFade(n + 1)),
            Self::EffectsAdjust(n, m) => Some(Self::EffectsAdjust(*n, m + 1)),
            Self::EffectsPos(n) => Some(Self::EffectsPos(n + 1)),
            Self::EffectsPosRotate(n) => Some(Self::EffectsPosRotate(n + 1)),
            Self::Focus(n) => Some(Self::Focus(n + 1)),
            Self::FocusAdjust(n) => Some(Self::FocusAdjust(n + 1)),
            Self::FocusDistance(n) => Some(Self::FocusDistance(n + 1)),
            Self::Control(n) => Some(Self::Control(n + 1)),
            Self::GoboWheelMode(n) => Some(Self::GoboWheelMode(n + 1)),
            Self::AnimationWheelMode(n) => Some(Self::AnimationWheelMode(n + 1)),
            Self::ColorMode(n) => Some(Self::ColorMode(n + 1)),
            Self::FanMode(n) => Some(Self::FanMode(n + 1)),
            Self::GoboWheelMSpeed(n) => Some(Self::GoboWheelMSpeed(n + 1)),
            Self::PrismMSpeed(n) => Some(Self::PrismMSpeed(n + 1)),
            Self::FrostMSpeed(n) => Some(Self::FrostMSpeed(n + 1)),
            Self::Blower(n) => Some(Self::Blower(n + 1)),
            Self::Fan(n) => Some(Self::Fan(n + 1)),
            Self::Fog(n) => Some(Self::Fog(n + 1)),
            Self::Haze(n) => Some(Self::Haze(n + 1)),
            Self::BladeA(n) => Some(Self::BladeA(n + 1)),
            Self::BladeB(n) => Some(Self::BladeB(n + 1)),
            Self::BladeRot(n) => Some(Self::BladeRot(n + 1)),
            Self::BladeSoftA(n) => Some(Self::BladeSoftA(n + 1)),
            Self::BladeSoftB(n) => Some(Self::BladeSoftB(n + 1)),
            Self::KeyStoneA(n) => Some(Self::KeyStoneA(n + 1)),
            Self::KeyStoneB(n) => Some(Self::KeyStoneB(n + 1)),
            Self::VideoEffectType(n) => Some(Self::VideoEffectType(n + 1)),
            Self::VideoEffectParameter(n, m) => Some(Self::VideoEffectParameter(*n, m + 1)),
            Self::VideoCamera(n) => Some(Self::VideoCamera(n + 1)),
            Self::VideoSoundVolume(n) => Some(Self::VideoSoundVolume(n + 1)),
            _ => None,
        }
    }

    /// Get a pretty name of the attribute.
    pub fn pretty(&self) -> String {
        match self {
            Self::Dimmer => "Dimmer".to_string(),

            Self::Pan => "Pan".to_string(),
            Self::Tilt => "Tilt".to_string(),
            Self::PanRotate => "Pan Rotate".to_string(),
            Self::TiltRotate => "Tilt Rotate".to_string(),
            Self::PositionEffect => "Position Effect".to_string(),
            Self::PositionEffectRate => "Position Effect Rate".to_string(),
            Self::PositionEffectFade => "Position Effect Fade".to_string(),
            Self::XyzX => "X".to_string(),
            Self::XyzY => "Y".to_string(),
            Self::XyzZ => "Z".to_string(),
            Self::RotX => "Rotation X".to_string(),
            Self::RotY => "Rotation Y".to_string(),
            Self::RotZ => "Rotation Z".to_string(),
            Self::ScaleX => "Scale X".to_string(),
            Self::ScaleY => "Scale Y".to_string(),
            Self::ScaleZ => "Scale Z".to_string(),
            Self::ScaleXYZ => "Scale XYZ".to_string(),

            Self::Gobo(n) => format!("Gobo {n}"),
            Self::GoboSelectSpin(n) => format!("Gobo {n} Select Spin"),
            Self::GoboSelectShake(n) => format!("Gobo {n} Select Shake"),
            Self::GoboSelectEffects(n) => format!("Gobo {n} Select Effects"),
            Self::GoboWheelIndex(n) => format!("Gobo {n} Wheel Index"),
            Self::GoboWheelSpin(n) => format!("Gobo {n} Wheel Spin"),
            Self::GoboWheelShake(n) => format!("Gobo {n} Wheel Shake"),
            Self::GoboWheelRandom(n) => format!("Gobo {n} Wheel Random"),
            Self::GoboWheelAudio(n) => format!("Gobo {n} Wheel Audio"),
            Self::GoboPos(n) => format!("Gobo {n} Position"),
            Self::GoboPosRotate(n) => format!("Gobo {n} Position Rotate"),
            Self::GoboPosShake(n) => format!("Gobo {n} Position Shake"),
            Self::AnimationWheel(n) => format!("Animation Wheel {n}"),
            Self::AnimationWheelAudio(n) => format!("Animation Wheel {n} Audio"),
            Self::AnimationWheelMacro(n) => format!("Animation Wheel {n} Macro"),
            Self::AnimationWheelRandom(n) => format!("Animation Wheel {n} Random"),
            Self::AnimationWheelSelectEffects(n) => format!("Animation Wheel {n} Select Effects"),
            Self::AnimationWheelSelectShake(n) => format!("Animation Wheel {n} Select Shake"),
            Self::AnimationWheelSelectSpin(n) => format!("Animation Wheel {n} Select Spin"),
            Self::AnimationWheelPos(n) => format!("Animation Wheel {n} Position"),
            Self::AnimationWheelPosRotate(n) => format!("Animation Wheel {n} Position Rotate"),
            Self::AnimationWheelPosShake(n) => format!("Animation Wheel {n} Position Shake"),
            Self::AnimationSystem(n) => format!("Animation System {n}"),
            Self::AnimationSystemRamp(n) => format!("Animation System {n} Ramp"),
            Self::AnimationSystemShake(n) => format!("Animation System {n} Shake"),
            Self::AnimationSystemAudio(n) => format!("Animation System {n} Audio"),
            Self::AnimationSystemRandom(n) => format!("Animation System {n} Random"),
            Self::AnimationSystemPos(n) => format!("Animation System {n} Position"),
            Self::AnimationSystemPosRotate(n) => format!("Animation System {n} Position Rotate"),
            Self::AnimationSystemPosShake(n) => format!("Animation System {n} Position Shake"),
            Self::AnimationSystemPosRandom(n) => format!("Animation System {n} Position Random"),
            Self::AnimationSystemPosAudio(n) => format!("Animation System {n} Position Audio"),
            Self::AnimationSystemMacro(n) => format!("Animation System {n} Macro"),
            Self::MediaFolder(n) => format!("Media Folder {n}"),
            Self::MediaContent(n) => format!("Media Content {n}"),
            Self::ModelFolder(n) => format!("Model Folder {n}"),
            Self::ModelContent(n) => format!("Model Content {n}"),
            Self::PlayMode => "Play Mode".to_string(),
            Self::PlayBegin => "Play Begin".to_string(),
            Self::PlayEnd => "Play End".to_string(),
            Self::PlaySpeed => "Play Speed".to_string(),

            Self::ColorEffects(n) => format!("Color Effects {n}"),
            Self::Color(n) => format!("Color {n}"),
            Self::ColorWheelIndex(n) => format!("Color Wheel {n} Index"),
            Self::ColorWheelSpin(n) => format!("Color Wheel {n} Spin"),
            Self::ColorWheelRandom(n) => format!("Color Wheel {n} Random"),
            Self::ColorWheelAudio(n) => format!("Color Wheel {n} Audio"),
            Self::ColorAddR => "Red".to_string(),
            Self::ColorAddG => "Green".to_string(),
            Self::ColorAddB => "Blue".to_string(),
            Self::ColorAddC => "Cyan".to_string(),
            Self::ColorAddM => "Magenta".to_string(),
            Self::ColorAddY => "Yellow".to_string(),
            Self::ColorAddRY => "Red-Yellow".to_string(),
            Self::ColorAddGY => "Green-Yellow".to_string(),
            Self::ColorAddGC => "Green-Cyan".to_string(),
            Self::ColorAddBC => "Blue-Cyan".to_string(),
            Self::ColorAddBM => "Blue-Magenta".to_string(),
            Self::ColorAddRM => "Red-Magenta".to_string(),
            Self::ColorAddW => "White".to_string(),
            Self::ColorAddWW => "Warm White".to_string(),
            Self::ColorAddCW => "Cool White".to_string(),
            Self::ColorAddUV => "UV".to_string(),
            Self::ColorSubR => "Red-".to_string(),
            Self::ColorSubG => "Green-".to_string(),
            Self::ColorSubB => "Blue-".to_string(),
            Self::ColorSubC => "Cyan-".to_string(),
            Self::ColorSubM => "Magenta-".to_string(),
            Self::ColorSubY => "Yellow-".to_string(),
            Self::ColorMacro(n) => format!("Color Macro {n}"),
            Self::ColorMacroRate(n) => format!("Color Macro Rate {n}"),
            Self::Cto => "CTO".to_string(),
            Self::Ctc => "CTC".to_string(),
            Self::Ctb => "CTB".to_string(),
            Self::Tint => "Tint".to_string(),
            Self::HsbHue => "HSB Hue".to_string(),
            Self::HsbSaturation => "HSB Saturation".to_string(),
            Self::HsbBrightness => "HSB Brightness".to_string(),
            Self::HsbQuality => "HSB Quality".to_string(),
            Self::CieX => "CIE X".to_string(),
            Self::CieY => "CIE Y".to_string(),
            Self::CieBrightness => "CIE Brightness".to_string(),
            Self::ColorRgbRed => "RGB Red".to_string(),
            Self::ColorRgbGreen => "RGB Green".to_string(),
            Self::ColorRgbBlue => "RGB Blue".to_string(),
            Self::ColorRgbCyan => "RGB Cyan".to_string(),
            Self::ColorRgbMagenta => "RGB Magenta".to_string(),
            Self::ColorRgbYellow => "RGB Yellow".to_string(),
            Self::ColorRgbQuality => "RGB Quality".to_string(),
            Self::VideoBoostR => "Video Boost Red".to_string(),
            Self::VideoBoostG => "Video Boost Green".to_string(),
            Self::VideoBoostB => "Video Boost Blue".to_string(),
            Self::VideoHueShift => "Video Hue Shift".to_string(),
            Self::VideoSaturation => "Video Saturation".to_string(),
            Self::VideoBrightness => "Video Brightness".to_string(),
            Self::VideoContrast => "Video Contrast".to_string(),
            Self::VideoKeyColorR => "Video Key Color Red".to_string(),
            Self::VideoKeyColorG => "Video Key Color Green".to_string(),
            Self::VideoKeyColorB => "Video Key Color Blue".to_string(),
            Self::VideoKeyIntensity => "Video Key Intensity".to_string(),
            Self::VideoKeyTolerance => "Video Key Tolerance".to_string(),

            Self::StrobeDuration => "Strobe Duration".to_string(),
            Self::StrobeRate => "Strobe Rate".to_string(),
            Self::StrobeFrequency => "Strobe Frequency".to_string(),
            Self::StrobeModeShutter => "Strobe Mode Shutter".to_string(),
            Self::StrobeModeStrobe => "Strobe Mode Strobe".to_string(),
            Self::StrobeModePulse => "Strobe Mode Pulse".to_string(),
            Self::StrobeModePulseOpen => "Strobe Mode Pulse Open".to_string(),
            Self::StrobeModePulseClose => "Strobe Mode Pulse Close".to_string(),
            Self::StrobeModeRandom => "Strobe Mode Random".to_string(),
            Self::StrobeModeRandomPulse => "Strobe Mode Random Pulse".to_string(),
            Self::StrobeModeRandomPulseOpen => "Strobe Mode Random Pulse Open".to_string(),
            Self::StrobeModeRandomPulseClose => "Strobe Mode Random Pulse Close".to_string(),
            Self::StrobeModeEffect => "Strobe Mode Effect".to_string(),
            Self::Shutter(n) => format!("Shutter {n}"),
            Self::ShutterStrobe(n) => format!("Shutter {n} Strobe"),
            Self::ShutterStrobePulse(n) => format!("Shutter {n} Strobe Pulse"),
            Self::ShutterStrobePulseClose(n) => format!("Shutter {n} Strobe Pulse Close"),
            Self::ShutterStrobePulseOpen(n) => format!("Shutter {n} Strobe Pulse Open"),
            Self::ShutterStrobeRandom(n) => format!("Shutter {n} Strobe Random"),
            Self::ShutterStrobeRandomPulse(n) => format!("Shutter {n} Strobe Random Pulse"),
            Self::ShutterStrobeRandomPulseClose(n) => {
                format!("Shutter {n} Strobe Random Pulse Close")
            }
            Self::ShutterStrobeRandomPulseOpen(n) => {
                format!("Shutter {n} Strobe Random Pulse Open")
            }
            Self::ShutterStrobeEffect(n) => format!("Shutter {n} Strobe Effect"),
            Self::Iris => "Iris".to_string(),
            Self::IrisStrobe => "Iris Strobe".to_string(),
            Self::IrisStrobeRandom => "Iris Strobe Random".to_string(),
            Self::IrisPulseClose => "Iris Pulse Close".to_string(),
            Self::IrisPulseOpen => "Iris Pulse Open".to_string(),
            Self::IrisRandomPulseClose => "Iris Random Pulse Close".to_string(),
            Self::IrisRandomPulseOpen => "Iris Random Pulse Open".to_string(),
            Self::Frost(n) => format!("Frost {n}"),
            Self::FrostPulseOpen(n) => format!("Frost {n} Pulse Open"),
            Self::FrostPulseClose(n) => format!("Frost {n} Pulse Close"),
            Self::FrostRamp(n) => format!("Frost {n} Ramp"),
            Self::Prism(n) => format!("Prism {n}"),
            Self::PrismSelectSpin(n) => format!("Prism {n} Select Spin"),
            Self::PrismMacro(n) => format!("Prism {n} Macro"),
            Self::PrismPos(n) => format!("Prism {n} Position"),
            Self::PrismPosRotate(n) => format!("Prism {n} Position Rotate"),

            Self::Effects(n) => format!("Effects {n}"),
            Self::EffectsRate(n) => format!("Effects {n} Rate"),
            Self::EffectsFade(n) => format!("Effects {n} Fade"),
            Self::EffectsAdjust(n, m) => format!("Effects {n} Adjust {m}"),
            Self::EffectsPos(n) => format!("Effects {n} Position"),
            Self::EffectsPosRotate(n) => format!("Effects {n} Position Rotate"),
            Self::EffectsSync => "Effects Sync".to_string(),
            Self::BeamShaper => "Beam Shaper".to_string(),
            Self::BeamShaperMacro => "Beam Shaper Macro".to_string(),
            Self::BeamShaperPos => "Beam Shaper Position".to_string(),
            Self::BeamShaperPosRotate => "Beam Shaper Position Rotate".to_string(),
            Self::Zoom => "Zoom".to_string(),
            Self::ZoomModeSpot => "Zoom Mode Spot".to_string(),
            Self::ZoomModeBeam => "Zoom Mode Beam".to_string(),
            Self::DigitalZoom => "Digital Zoom".to_string(),
            Self::Focus(n) => format!("Focus {n}"),
            Self::FocusAdjust(n) => format!("Focus {n} Adjust"),
            Self::FocusDistance(n) => format!("Focus {n} Distance"),

            Self::Control(n) => format!("Control {n}"),
            Self::DimmerMode => "Dimmer Mode".to_string(),
            Self::DimmerCurve => "Dimmer Curve".to_string(),
            Self::BlackoutMode => "Blackout Mode".to_string(),
            Self::LedFrequency => "LED Frequency".to_string(),
            Self::LedZoneMode => "LED Zone Mode".to_string(),
            Self::PixelMode => "Pixel Mode".to_string(),
            Self::PanMode => "Pan Mode".to_string(),
            Self::TiltMode => "Tilt Mode".to_string(),
            Self::PanTiltMode => "Pan Tilt Mode".to_string(),
            Self::PositionModes => "Position Modes".to_string(),
            Self::GoboWheelMode(n) => format!("Gobo {n} Wheel Mode"),
            Self::GoboWheelShortcutMode => "Gobo Wheel Shortcut Mode".to_string(),
            Self::AnimationWheelMode(n) => format!("Animation {n} Wheel Mode"),
            Self::AnimationWheelShortcutMode => "Animation Wheel Shortcut Mode".to_string(),
            Self::ColorMode(n) => format!("Color {n} Mode"),
            Self::ColorWheelShortcutMode => "Color Wheel Shortcut Mode".to_string(),
            Self::CyanMode => "Cyan Mode".to_string(),
            Self::MagentaMode => "Magenta Mode".to_string(),
            Self::YellowMode => "Yellow Mode".to_string(),
            Self::ColorMixMode => "Color Mix Mode".to_string(),
            Self::ChromaticMode => "Chromatic Mode".to_string(),
            Self::ColorCalibrationMode => "Color Calibration Mode".to_string(),
            Self::ColorConsistency => "Color Consistency".to_string(),
            Self::ColorControl => "Color Control".to_string(),
            Self::ColorModelMode => "Color Model Mode".to_string(),
            Self::ColorSettingsReset => "Color Settings Reset".to_string(),
            Self::ColorUniformity => "Color Uniformity".to_string(),
            Self::CriMode => "CRI Mode".to_string(),
            Self::CustomColor => "Custom Color".to_string(),
            Self::UvStability => "UV Stability".to_string(),
            Self::WavelengthCorrection => "Wavelength Correction".to_string(),
            Self::WhiteCount => "White Count".to_string(),
            Self::StrobeMode => "Strobe Mode".to_string(),
            Self::ZoomMode => "Zoom Mode".to_string(),
            Self::FocusMode => "Focus Mode".to_string(),
            Self::IrisMode => "Iris Mode".to_string(),
            Self::FanMode(n) => format!("Fan {n} Mode"),
            Self::FollowSpotMode => "Follow Spot Mode".to_string(),
            Self::BeamEffectIndexRotateMode => "Beam Effect Index Rotate Mode".to_string(),
            Self::IntensityMSpeed => "Intensity MSpeed".to_string(),
            Self::PositionMSpeed => "Position MSpeed".to_string(),
            Self::ColorMixMSpeed => "Color Mix MSpeed".to_string(),
            Self::ColorWheelSelectMSpeed => "Color Wheel Select MSpeed".to_string(),
            Self::GoboWheelMSpeed(n) => format!("Gobo {n} Wheel MSpeed"),
            Self::IrisMSpeed => "Iris MSpeed".to_string(),
            Self::PrismMSpeed(n) => format!("Prism {n} MSpeed"),
            Self::FocusMSpeed => "Focus MSpeed".to_string(),
            Self::FrostMSpeed(n) => format!("Frost {n} MSpeed"),
            Self::ZoomMSpeed => "Zoom MSpeed".to_string(),
            Self::FrameMSpeed => "Frame MSpeed".to_string(),
            Self::GlobalMSpeed => "Global MSpeed".to_string(),
            Self::ReflectorAdjust => "Reflector Adjust".to_string(),
            Self::FixtureGlobalReset => "Fixture Global Reset".to_string(),
            Self::DimmerReset => "Dimmer Reset".to_string(),
            Self::ShutterReset => "Shutter Reset".to_string(),
            Self::BeamReset => "Beam Reset".to_string(),
            Self::ColorMixReset => "Color Mix Reset".to_string(),
            Self::ColorWheelReset => "Color Wheel Reset".to_string(),
            Self::FocusReset => "Focus Reset".to_string(),
            Self::FrameReset => "Frame Reset".to_string(),
            Self::GoboWheelReset => "Gobo Wheel Reset".to_string(),
            Self::IntensityReset => "Intensity Reset".to_string(),
            Self::IrisReset => "Iris Reset".to_string(),
            Self::PositionReset => "Position Reset".to_string(),
            Self::PanReset => "Pan Reset".to_string(),
            Self::TiltReset => "Tilt Reset".to_string(),
            Self::ZoomReset => "Zoom Reset".to_string(),
            Self::CtbReset => "CTB Reset".to_string(),
            Self::CtoReset => "CTO Reset".to_string(),
            Self::CtcReset => "CTC Reset".to_string(),
            Self::AnimationSystemReset => "Animation System Reset".to_string(),
            Self::FixtureCalibrationReset => "Fixture Calibration Reset".to_string(),
            Self::Function => "Function".to_string(),
            Self::LampControl => "Lamp Control".to_string(),
            Self::DisplayIntensity => "Display Intensity".to_string(),
            Self::DmxInput => "DMX Input".to_string(),
            Self::NoFeature => "No Feature".to_string(),
            Self::Blower(n) => format!("Blower {n}"),
            Self::Fan(n) => format!("Fan {n}"),
            Self::Fog(n) => format!("Fog {n}"),
            Self::Haze(n) => format!("Haze {n}"),
            Self::LampPowerMode => "Lamp Power Mode".to_string(),
            Self::Fans => "Fans".to_string(),

            Self::BladeA(n) => format!("Blade {n} A"),
            Self::BladeB(n) => format!("Blade {n} B"),
            Self::BladeRot(n) => format!("Blade {n} Rotation"),
            Self::ShaperRot => "Shaper Rotation".to_string(),
            Self::ShaperMacros => "Shaper Macros".to_string(),
            Self::ShaperMacrosSpeed => "Shaper Macros Speed".to_string(),
            Self::BladeSoftA(n) => format!("Blade Soft {n} A"),
            Self::BladeSoftB(n) => format!("Blade Soft {n} B"),
            Self::KeyStoneA(n) => format!("Keystone {n} A"),
            Self::KeyStoneB(n) => format!("Keystone {n} B"),

            Self::Video => "Video".to_string(),
            Self::VideoEffectType(n) => format!("Video Effect {n} Type"),
            Self::VideoEffectParameter(n, m) => format!("Video Effect {n} Parameter {m}"),
            Self::VideoCamera(n) => format!("Video Camera {n}"),
            Self::VideoSoundVolume(n) => format!("Video Sound Volume {n}"),
            Self::VideoBlendMode => "Video Blend Mode".to_string(),
            Self::InputSource => "Input Source".to_string(),
            Self::FieldOfView => "Field of View".to_string(),

            Self::Custom(name) => name.to_string(),
        }
    }

    pub fn feature_type(&self) -> Option<FixtureChannel3FeatureType> {
        match self {
            Self::Dimmer => Some(FixtureChannel3FeatureType::Dimmer),
            Self::Pan => Some(FixtureChannel3FeatureType::PanTilt),
            Self::Tilt => Some(FixtureChannel3FeatureType::PanTilt),
            Self::PanRotate => Some(FixtureChannel3FeatureType::PanTilt),
            Self::TiltRotate => Some(FixtureChannel3FeatureType::PanTilt),
            Self::PositionEffect => Some(FixtureChannel3FeatureType::PanTilt),
            Self::PositionEffectRate => Some(FixtureChannel3FeatureType::PanTilt),
            Self::PositionEffectFade => Some(FixtureChannel3FeatureType::PanTilt),
            Self::XyzX => Some(FixtureChannel3FeatureType::Xyz),
            Self::XyzY => Some(FixtureChannel3FeatureType::Xyz),
            Self::XyzZ => Some(FixtureChannel3FeatureType::Xyz),
            Self::RotX => Some(FixtureChannel3FeatureType::Rotation),
            Self::RotY => Some(FixtureChannel3FeatureType::Rotation),
            Self::RotZ => Some(FixtureChannel3FeatureType::Rotation),
            Self::ScaleX => Some(FixtureChannel3FeatureType::Scale),
            Self::ScaleY => Some(FixtureChannel3FeatureType::Scale),
            Self::ScaleZ => Some(FixtureChannel3FeatureType::Scale),
            Self::ScaleXYZ => Some(FixtureChannel3FeatureType::Scale),
            Self::Gobo(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboSelectSpin(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboSelectShake(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboSelectEffects(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboWheelIndex(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboWheelSpin(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboWheelShake(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboWheelRandom(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboWheelAudio(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboPos(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboPosRotate(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::GoboPosShake(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheel(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheelAudio(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheelMacro(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheelRandom(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheelSelectEffects(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheelSelectShake(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheelSelectSpin(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheelPos(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheelPosRotate(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationWheelPosShake(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystem(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemRamp(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemShake(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemAudio(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemRandom(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemPos(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemPosRotate(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemPosShake(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemPosRandom(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemPosAudio(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::AnimationSystemMacro(_) => Some(FixtureChannel3FeatureType::Gobo),
            Self::PlayMode => Some(FixtureChannel3FeatureType::Media),
            Self::MediaFolder(_) => Some(FixtureChannel3FeatureType::Media),
            Self::MediaContent(_) => Some(FixtureChannel3FeatureType::Media),
            Self::ModelFolder(_) => Some(FixtureChannel3FeatureType::Media),
            Self::ModelContent(_) => Some(FixtureChannel3FeatureType::Media),
            Self::PlayBegin => Some(FixtureChannel3FeatureType::Media),
            Self::PlayEnd => Some(FixtureChannel3FeatureType::Media),
            Self::PlaySpeed => Some(FixtureChannel3FeatureType::Media),
            Self::ColorEffects(_) => Some(FixtureChannel3FeatureType::Color),
            Self::Color(_) => Some(FixtureChannel3FeatureType::Color),
            Self::ColorWheelIndex(_) => Some(FixtureChannel3FeatureType::Color),
            Self::ColorWheelSpin(_) => Some(FixtureChannel3FeatureType::Color),
            Self::ColorWheelRandom(_) => Some(FixtureChannel3FeatureType::Color),
            Self::ColorWheelAudio(_) => Some(FixtureChannel3FeatureType::Color),
            Self::ColorAddR => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddG => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddB => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddC => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddM => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddY => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddRY => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddGY => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddGC => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddBC => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddBM => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddRM => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddW => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddWW => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddCW => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorAddUV => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorSubR => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorSubG => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorSubB => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorSubC => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorSubM => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorSubY => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorMacro(_) => Some(FixtureChannel3FeatureType::Rgb),
            Self::ColorMacroRate(_) => Some(FixtureChannel3FeatureType::Rgb),
            Self::Cto => Some(FixtureChannel3FeatureType::Color),
            Self::Ctc => Some(FixtureChannel3FeatureType::Color),
            Self::Ctb => Some(FixtureChannel3FeatureType::Color),
            Self::Tint => Some(FixtureChannel3FeatureType::Color),
            Self::HsbHue => Some(FixtureChannel3FeatureType::Hsb),
            Self::HsbSaturation => Some(FixtureChannel3FeatureType::Hsb),
            Self::HsbBrightness => Some(FixtureChannel3FeatureType::Hsb),
            Self::HsbQuality => Some(FixtureChannel3FeatureType::Hsb),
            Self::CieX => Some(FixtureChannel3FeatureType::Cie),
            Self::CieY => Some(FixtureChannel3FeatureType::Cie),
            Self::CieBrightness => Some(FixtureChannel3FeatureType::Cie),
            Self::ColorRgbRed => Some(FixtureChannel3FeatureType::Indirect),
            Self::ColorRgbGreen => Some(FixtureChannel3FeatureType::Indirect),
            Self::ColorRgbBlue => Some(FixtureChannel3FeatureType::Indirect),
            Self::ColorRgbCyan => Some(FixtureChannel3FeatureType::Indirect),
            Self::ColorRgbMagenta => Some(FixtureChannel3FeatureType::Indirect),
            Self::ColorRgbYellow => Some(FixtureChannel3FeatureType::Indirect),
            Self::ColorRgbQuality => Some(FixtureChannel3FeatureType::Indirect),
            Self::VideoBoostR => Some(FixtureChannel3FeatureType::ColorCorrection),
            Self::VideoBoostG => Some(FixtureChannel3FeatureType::ColorCorrection),
            Self::VideoBoostB => Some(FixtureChannel3FeatureType::ColorCorrection),
            Self::VideoHueShift => Some(FixtureChannel3FeatureType::HsbcShift),
            Self::VideoSaturation => Some(FixtureChannel3FeatureType::HsbcShift),
            Self::VideoBrightness => Some(FixtureChannel3FeatureType::HsbcShift),
            Self::VideoContrast => Some(FixtureChannel3FeatureType::HsbcShift),
            Self::VideoKeyColorR => Some(FixtureChannel3FeatureType::ColorKey),
            Self::VideoKeyColorG => Some(FixtureChannel3FeatureType::ColorKey),
            Self::VideoKeyColorB => Some(FixtureChannel3FeatureType::ColorKey),
            Self::VideoKeyIntensity => Some(FixtureChannel3FeatureType::ColorKey),
            Self::VideoKeyTolerance => Some(FixtureChannel3FeatureType::ColorKey),
            Self::StrobeDuration => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeRate => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeFrequency => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModeShutter => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModeStrobe => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModePulse => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModePulseOpen => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModePulseClose => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModeRandom => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModeRandomPulse => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModeRandomPulseOpen => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModeRandomPulseClose => Some(FixtureChannel3FeatureType::Beam),
            Self::StrobeModeEffect => Some(FixtureChannel3FeatureType::Beam),
            Self::Shutter(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::ShutterStrobe(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::ShutterStrobePulse(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::ShutterStrobePulseClose(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::ShutterStrobePulseOpen(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::ShutterStrobeRandom(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::ShutterStrobeRandomPulse(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::ShutterStrobeRandomPulseClose(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::ShutterStrobeRandomPulseOpen(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::ShutterStrobeEffect(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::Iris => Some(FixtureChannel3FeatureType::Beam),
            Self::IrisStrobe => Some(FixtureChannel3FeatureType::Beam),
            Self::IrisStrobeRandom => Some(FixtureChannel3FeatureType::Beam),
            Self::IrisPulseClose => Some(FixtureChannel3FeatureType::Beam),
            Self::IrisPulseOpen => Some(FixtureChannel3FeatureType::Beam),
            Self::IrisRandomPulseClose => Some(FixtureChannel3FeatureType::Beam),
            Self::IrisRandomPulseOpen => Some(FixtureChannel3FeatureType::Beam),
            Self::Frost(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::FrostPulseOpen(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::FrostPulseClose(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::FrostRamp(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::Prism(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::PrismSelectSpin(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::PrismMacro(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::PrismPos(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::PrismPosRotate(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::Effects(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::EffectsRate(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::EffectsFade(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::EffectsAdjust(_, _) => Some(FixtureChannel3FeatureType::Beam),
            Self::EffectsPos(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::EffectsPosRotate(_) => Some(FixtureChannel3FeatureType::Beam),
            Self::EffectsSync => Some(FixtureChannel3FeatureType::Beam),
            Self::BeamShaper => Some(FixtureChannel3FeatureType::Beam),
            Self::BeamShaperMacro => Some(FixtureChannel3FeatureType::Beam),
            Self::BeamShaperPos => Some(FixtureChannel3FeatureType::Beam),
            Self::BeamShaperPosRotate => Some(FixtureChannel3FeatureType::Beam),
            Self::Zoom => Some(FixtureChannel3FeatureType::Focus),
            Self::ZoomModeSpot => Some(FixtureChannel3FeatureType::Focus),
            Self::ZoomModeBeam => Some(FixtureChannel3FeatureType::Focus),
            Self::DigitalZoom => Some(FixtureChannel3FeatureType::Focus),
            Self::Focus(_) => Some(FixtureChannel3FeatureType::Focus),
            Self::FocusAdjust(_) => Some(FixtureChannel3FeatureType::Focus),
            Self::FocusDistance(_) => Some(FixtureChannel3FeatureType::Focus),
            Self::Control(_) => Some(FixtureChannel3FeatureType::Control),
            Self::DimmerMode => Some(FixtureChannel3FeatureType::Control),
            Self::DimmerCurve => Some(FixtureChannel3FeatureType::Control),
            Self::BlackoutMode => Some(FixtureChannel3FeatureType::Control),
            Self::LedFrequency => Some(FixtureChannel3FeatureType::Control),
            Self::LedZoneMode => Some(FixtureChannel3FeatureType::Control),
            Self::PixelMode => Some(FixtureChannel3FeatureType::Control),
            Self::PanMode => Some(FixtureChannel3FeatureType::Control),
            Self::TiltMode => Some(FixtureChannel3FeatureType::Control),
            Self::PanTiltMode => Some(FixtureChannel3FeatureType::Control),
            Self::PositionModes => Some(FixtureChannel3FeatureType::Control),
            Self::GoboWheelMode(_) => Some(FixtureChannel3FeatureType::Control),
            Self::GoboWheelShortcutMode => Some(FixtureChannel3FeatureType::Control),
            Self::AnimationWheelMode(_) => Some(FixtureChannel3FeatureType::Control),
            Self::AnimationWheelShortcutMode => Some(FixtureChannel3FeatureType::Control),
            Self::ColorMode(_) => Some(FixtureChannel3FeatureType::Control),
            Self::ColorWheelShortcutMode => Some(FixtureChannel3FeatureType::Control),
            Self::CyanMode => Some(FixtureChannel3FeatureType::Control),
            Self::MagentaMode => Some(FixtureChannel3FeatureType::Control),
            Self::YellowMode => Some(FixtureChannel3FeatureType::Control),
            Self::ColorMixMode => Some(FixtureChannel3FeatureType::Control),
            Self::ChromaticMode => Some(FixtureChannel3FeatureType::Control),
            Self::ColorCalibrationMode => Some(FixtureChannel3FeatureType::Control),
            Self::ColorConsistency => Some(FixtureChannel3FeatureType::Control),
            Self::ColorControl => Some(FixtureChannel3FeatureType::Control),
            Self::ColorModelMode => Some(FixtureChannel3FeatureType::Control),
            Self::ColorSettingsReset => Some(FixtureChannel3FeatureType::Control),
            Self::ColorUniformity => Some(FixtureChannel3FeatureType::Control),
            Self::CriMode => Some(FixtureChannel3FeatureType::Control),
            Self::CustomColor => Some(FixtureChannel3FeatureType::Control),
            Self::UvStability => Some(FixtureChannel3FeatureType::Control),
            Self::WavelengthCorrection => Some(FixtureChannel3FeatureType::Control),
            Self::WhiteCount => Some(FixtureChannel3FeatureType::Control),
            Self::StrobeMode => Some(FixtureChannel3FeatureType::Control),
            Self::ZoomMode => Some(FixtureChannel3FeatureType::Control),
            Self::FocusMode => Some(FixtureChannel3FeatureType::Control),
            Self::IrisMode => Some(FixtureChannel3FeatureType::Control),
            Self::FanMode(_) => Some(FixtureChannel3FeatureType::Control),
            Self::FollowSpotMode => Some(FixtureChannel3FeatureType::Control),
            Self::BeamEffectIndexRotateMode => Some(FixtureChannel3FeatureType::Control),
            Self::IntensityMSpeed => Some(FixtureChannel3FeatureType::Control),
            Self::PositionMSpeed => Some(FixtureChannel3FeatureType::Control),
            Self::ColorMixMSpeed => Some(FixtureChannel3FeatureType::Control),
            Self::ColorWheelSelectMSpeed => Some(FixtureChannel3FeatureType::Control),
            Self::GoboWheelMSpeed(_) => Some(FixtureChannel3FeatureType::Control),
            Self::IrisMSpeed => Some(FixtureChannel3FeatureType::Control),
            Self::PrismMSpeed(_) => Some(FixtureChannel3FeatureType::Control),
            Self::FocusMSpeed => Some(FixtureChannel3FeatureType::Control),
            Self::FrostMSpeed(_) => Some(FixtureChannel3FeatureType::Control),
            Self::ZoomMSpeed => Some(FixtureChannel3FeatureType::Control),
            Self::FrameMSpeed => Some(FixtureChannel3FeatureType::Control),
            Self::GlobalMSpeed => Some(FixtureChannel3FeatureType::Control),
            Self::ReflectorAdjust => Some(FixtureChannel3FeatureType::Control),
            Self::FixtureGlobalReset => Some(FixtureChannel3FeatureType::Control),
            Self::DimmerReset => Some(FixtureChannel3FeatureType::Control),
            Self::ShutterReset => Some(FixtureChannel3FeatureType::Control),
            Self::BeamReset => Some(FixtureChannel3FeatureType::Control),
            Self::ColorMixReset => Some(FixtureChannel3FeatureType::Control),
            Self::ColorWheelReset => Some(FixtureChannel3FeatureType::Control),
            Self::FocusReset => Some(FixtureChannel3FeatureType::Control),
            Self::FrameReset => Some(FixtureChannel3FeatureType::Control),
            Self::GoboWheelReset => Some(FixtureChannel3FeatureType::Control),
            Self::IntensityReset => Some(FixtureChannel3FeatureType::Control),
            Self::IrisReset => Some(FixtureChannel3FeatureType::Control),
            Self::PositionReset => Some(FixtureChannel3FeatureType::Control),
            Self::PanReset => Some(FixtureChannel3FeatureType::Control),
            Self::TiltReset => Some(FixtureChannel3FeatureType::Control),
            Self::ZoomReset => Some(FixtureChannel3FeatureType::Control),
            Self::CtbReset => Some(FixtureChannel3FeatureType::Control),
            Self::CtoReset => Some(FixtureChannel3FeatureType::Control),
            Self::CtcReset => Some(FixtureChannel3FeatureType::Control),
            Self::AnimationSystemReset => Some(FixtureChannel3FeatureType::Control),
            Self::FixtureCalibrationReset => Some(FixtureChannel3FeatureType::Control),
            Self::Function => Some(FixtureChannel3FeatureType::Control),
            Self::LampControl => Some(FixtureChannel3FeatureType::Control),
            Self::DisplayIntensity => Some(FixtureChannel3FeatureType::Control),
            Self::DmxInput => Some(FixtureChannel3FeatureType::Control),
            Self::NoFeature => Some(FixtureChannel3FeatureType::Control),
            Self::Blower(_) => Some(FixtureChannel3FeatureType::Control),
            Self::Fan(_) => Some(FixtureChannel3FeatureType::Control),
            Self::Fog(_) => Some(FixtureChannel3FeatureType::Control),
            Self::Haze(_) => Some(FixtureChannel3FeatureType::Control),
            Self::LampPowerMode => Some(FixtureChannel3FeatureType::Control),
            Self::Fans => Some(FixtureChannel3FeatureType::Control),
            Self::BladeA(_) => Some(FixtureChannel3FeatureType::Shapers),
            Self::BladeB(_) => Some(FixtureChannel3FeatureType::Shapers),
            Self::BladeRot(_) => Some(FixtureChannel3FeatureType::Shapers),
            Self::ShaperRot => Some(FixtureChannel3FeatureType::Shapers),
            Self::ShaperMacros => Some(FixtureChannel3FeatureType::Shapers),
            Self::ShaperMacrosSpeed => Some(FixtureChannel3FeatureType::Shapers),
            Self::BladeSoftA(_) => Some(FixtureChannel3FeatureType::Shapers),
            Self::BladeSoftB(_) => Some(FixtureChannel3FeatureType::Shapers),
            Self::KeyStoneA(_) => Some(FixtureChannel3FeatureType::Shapers),
            Self::KeyStoneB(_) => Some(FixtureChannel3FeatureType::Shapers),
            Self::Video => Some(FixtureChannel3FeatureType::Video),
            Self::VideoEffectType(_) => Some(FixtureChannel3FeatureType::Video),
            Self::VideoEffectParameter(_, _) => Some(FixtureChannel3FeatureType::Video),
            Self::VideoCamera(_) => Some(FixtureChannel3FeatureType::Video),
            Self::FieldOfView => Some(FixtureChannel3FeatureType::Video),
            Self::InputSource => Some(FixtureChannel3FeatureType::Video),
            Self::VideoBlendMode => Some(FixtureChannel3FeatureType::Video),
            Self::VideoSoundVolume(_) => Some(FixtureChannel3FeatureType::Video),
            Self::Custom(_) => None,
        }
    }
}

impl fmt::Display for FixtureChannel3Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dimmer => write!(f, "Dimmer"),

            Self::Pan => write!(f, "Pan"),
            Self::Tilt => write!(f, "Tilt"),
            Self::PanRotate => write!(f, "PanRotate"),
            Self::TiltRotate => write!(f, "TiltRotate"),
            Self::PositionEffect => write!(f, "PositionEffect"),
            Self::PositionEffectRate => write!(f, "PositionEffectRate"),
            Self::PositionEffectFade => write!(f, "PositionEffectFade"),
            Self::XyzX => write!(f, "XYZ_X"),
            Self::XyzY => write!(f, "XYZ_Y"),
            Self::XyzZ => write!(f, "XYZ_Z"),
            Self::RotX => write!(f, "Rot_X"),
            Self::RotY => write!(f, "Rot_Y"),
            Self::RotZ => write!(f, "Rot_Z"),
            Self::ScaleX => write!(f, "Scale_X"),
            Self::ScaleY => write!(f, "Scale_Y"),
            Self::ScaleZ => write!(f, "Scale_Z"),
            Self::ScaleXYZ => write!(f, "Scale_XYZ"),

            Self::Gobo(n) => write!(f, "Gobo{n}"),
            Self::GoboSelectSpin(n) => write!(f, "Gobo{n}SelectSpin"),
            Self::GoboSelectShake(n) => write!(f, "Gobo{n}SelectShake"),
            Self::GoboSelectEffects(n) => write!(f, "Gobo{n}SelectEffects"),
            Self::GoboWheelIndex(n) => write!(f, "Gobo{n}WheelIndex"),
            Self::GoboWheelSpin(n) => write!(f, "Gobo{n}WheelSpin"),
            Self::GoboWheelShake(n) => write!(f, "Gobo{n}WheelShake"),
            Self::GoboWheelRandom(n) => write!(f, "Gobo{n}WheelRandom"),
            Self::GoboWheelAudio(n) => write!(f, "Gobo{n}WheelAudio"),
            Self::GoboPos(n) => write!(f, "Gobo{n}Pos"),
            Self::GoboPosRotate(n) => write!(f, "Gobo{n}PosRotate"),
            Self::GoboPosShake(n) => write!(f, "Gobo{n}PosShake"),
            Self::AnimationWheel(n) => write!(f, "AnimationWheel{n}"),
            Self::AnimationWheelAudio(n) => write!(f, "AnimationWheel{n}Audio"),
            Self::AnimationWheelMacro(n) => write!(f, "AnimationWheel{n}Macro"),
            Self::AnimationWheelRandom(n) => write!(f, "AnimationWheel{n}Random"),
            Self::AnimationWheelSelectEffects(n) => write!(f, "AnimationWheel{n}SelectEffects"),
            Self::AnimationWheelSelectShake(n) => write!(f, "AnimationWheel{n}SelectShake"),
            Self::AnimationWheelSelectSpin(n) => write!(f, "AnimationWheel{n}SelectSpin"),
            Self::AnimationWheelPos(n) => write!(f, "AnimationWheel{n}Pos"),
            Self::AnimationWheelPosRotate(n) => write!(f, "AnimationWheel{n}PosRotate"),
            Self::AnimationWheelPosShake(n) => write!(f, "AnimationWheel{n}PosShake"),
            Self::AnimationSystem(n) => write!(f, "AnimationSystem{n}"),
            Self::AnimationSystemRamp(n) => write!(f, "AnimationSystem{n}Ramp"),
            Self::AnimationSystemShake(n) => write!(f, "AnimationSystem{n}Shake"),
            Self::AnimationSystemAudio(n) => write!(f, "AnimationSystem{n}Audio"),
            Self::AnimationSystemRandom(n) => write!(f, "AnimationSystem{n}Random"),
            Self::AnimationSystemPos(n) => write!(f, "AnimationSystem{n}Pos"),
            Self::AnimationSystemPosRotate(n) => write!(f, "AnimationSystem{n}PosRotate"),
            Self::AnimationSystemPosShake(n) => write!(f, "AnimationSystem{n}PosShake"),
            Self::AnimationSystemPosRandom(n) => write!(f, "AnimationSystem{n}PosRandom"),
            Self::AnimationSystemPosAudio(n) => write!(f, "AnimationSystem{n}PosAudio"),
            Self::AnimationSystemMacro(n) => write!(f, "AnimationSystem{n}Macro"),
            Self::MediaFolder(n) => write!(f, "MediaFolder{n}"),
            Self::MediaContent(n) => write!(f, "MediaContent{n}"),
            Self::ModelFolder(n) => write!(f, "ModelFolder{n}"),
            Self::ModelContent(n) => write!(f, "ModelContent{n}"),
            Self::PlayMode => write!(f, "PlayMode"),
            Self::PlayBegin => write!(f, "PlayBegin"),
            Self::PlayEnd => write!(f, "PlayEnd"),
            Self::PlaySpeed => write!(f, "PlaySpeed"),

            Self::ColorEffects(n) => write!(f, "ColorEffects{n}"),
            Self::Color(n) => write!(f, "Color{n}"),
            Self::ColorWheelIndex(n) => write!(f, "ColorWheel{n}Index"),
            Self::ColorWheelSpin(n) => write!(f, "ColorWheel{n}Spin"),
            Self::ColorWheelRandom(n) => write!(f, "ColorWheel{n}Random"),
            Self::ColorWheelAudio(n) => write!(f, "ColorWheel{n}Audio"),
            Self::ColorAddR => write!(f, "ColorAdd_R"),
            Self::ColorAddG => write!(f, "ColorAdd_G"),
            Self::ColorAddB => write!(f, "ColorAdd_B"),
            Self::ColorAddC => write!(f, "ColorAdd_C"),
            Self::ColorAddM => write!(f, "ColorAdd_M"),
            Self::ColorAddY => write!(f, "ColorAdd_Y"),
            Self::ColorAddRY => write!(f, "ColorAdd_RY"),
            Self::ColorAddGY => write!(f, "ColorAdd_GY"),
            Self::ColorAddGC => write!(f, "ColorAdd_GC"),
            Self::ColorAddBC => write!(f, "ColorAdd_BC"),
            Self::ColorAddBM => write!(f, "ColorAdd_BM"),
            Self::ColorAddRM => write!(f, "ColorAdd_RM"),
            Self::ColorAddW => write!(f, "ColorAdd_W"),
            Self::ColorAddWW => write!(f, "ColorAdd_WW"),
            Self::ColorAddCW => write!(f, "ColorAdd_CW"),
            Self::ColorAddUV => write!(f, "ColorAdd_UV"),
            Self::ColorSubR => write!(f, "ColorSub_R"),
            Self::ColorSubG => write!(f, "ColorSub_G"),
            Self::ColorSubB => write!(f, "ColorSub_B"),
            Self::ColorSubC => write!(f, "ColorSub_C"),
            Self::ColorSubM => write!(f, "ColorSub_M"),
            Self::ColorSubY => write!(f, "ColorSub_Y"),
            Self::ColorMacro(n) => write!(f, "ColorMacro{n}"),
            Self::ColorMacroRate(n) => write!(f, "ColorMacroRate{n}"),
            Self::Cto => write!(f, "CTO"),
            Self::Ctc => write!(f, "CTC"),
            Self::Ctb => write!(f, "CTB"),
            Self::Tint => write!(f, "Tint"),
            Self::HsbHue => write!(f, "HSB_Hue"),
            Self::HsbSaturation => write!(f, "HSB_Saturation"),
            Self::HsbBrightness => write!(f, "HSB_Brightness"),
            Self::HsbQuality => write!(f, "HSB_Quality"),
            Self::CieX => write!(f, "CIE_X"),
            Self::CieY => write!(f, "CIE_Y"),
            Self::CieBrightness => write!(f, "CIE_Brightness"),
            Self::ColorRgbRed => write!(f, "ColorRGB_Red"),
            Self::ColorRgbGreen => write!(f, "ColorRGB_Green"),
            Self::ColorRgbBlue => write!(f, "ColorRGB_Blue"),
            Self::ColorRgbCyan => write!(f, "ColorRGB_Cyan"),
            Self::ColorRgbMagenta => write!(f, "ColorRGB_Magenta"),
            Self::ColorRgbYellow => write!(f, "ColorRGB_Yellow"),
            Self::ColorRgbQuality => write!(f, "ColorRGB_Quality"),
            Self::VideoBoostR => write!(f, "VideoBoost_R"),
            Self::VideoBoostG => write!(f, "VideoBoost_G"),
            Self::VideoBoostB => write!(f, "VideoBoost_B"),
            Self::VideoHueShift => write!(f, "VideoHueShift"),
            Self::VideoSaturation => write!(f, "VideoSaturation"),
            Self::VideoBrightness => write!(f, "VideoBrightness"),
            Self::VideoContrast => write!(f, "VideoContrast"),
            Self::VideoKeyColorR => write!(f, "VideoKeyColor_R"),
            Self::VideoKeyColorG => write!(f, "VideoKeyColor_G"),
            Self::VideoKeyColorB => write!(f, "VideoKeyColor_B"),
            Self::VideoKeyIntensity => write!(f, "VideoKeyIntensity"),
            Self::VideoKeyTolerance => write!(f, "VideoKeyTolerance"),

            Self::StrobeDuration => write!(f, "StrobeDuration"),
            Self::StrobeRate => write!(f, "StrobeRate"),
            Self::StrobeFrequency => write!(f, "StrobeFrequency"),
            Self::StrobeModeShutter => write!(f, "StrobeModeShutter"),
            Self::StrobeModeStrobe => write!(f, "StrobeModeStrobe"),
            Self::StrobeModePulse => write!(f, "StrobeModePulse"),
            Self::StrobeModePulseOpen => write!(f, "StrobeModePulseOpen"),
            Self::StrobeModePulseClose => write!(f, "StrobeModePulseClose"),
            Self::StrobeModeRandom => write!(f, "StrobeModeRandom"),
            Self::StrobeModeRandomPulse => write!(f, "StrobeModeRandomPulse"),
            Self::StrobeModeRandomPulseOpen => write!(f, "StrobeModeRandomPulseOpen"),
            Self::StrobeModeRandomPulseClose => write!(f, "StrobeModeRandomPulseClose"),
            Self::StrobeModeEffect => write!(f, "StrobeModeEffect"),
            Self::Shutter(n) => write!(f, "Shutter{n}"),
            Self::ShutterStrobe(n) => write!(f, "Shutter{n}Strobe"),
            Self::ShutterStrobePulse(n) => write!(f, "Shutter{n}StrobePulse"),
            Self::ShutterStrobePulseClose(n) => write!(f, "Shutter{n}StrobePulseClose"),
            Self::ShutterStrobePulseOpen(n) => write!(f, "Shutter{n}StrobePulseOpen"),
            Self::ShutterStrobeRandom(n) => write!(f, "Shutter{n}StrobeRandom"),
            Self::ShutterStrobeRandomPulse(n) => write!(f, "Shutter{n}StrobeRandomPulse"),
            Self::ShutterStrobeRandomPulseClose(n) => write!(f, "Shutter{n}StrobeRandomPulseClose"),
            Self::ShutterStrobeRandomPulseOpen(n) => write!(f, "Shutter{n}StrobeRandomPulseOpen"),
            Self::ShutterStrobeEffect(n) => write!(f, "Shutter{n}StrobeEffect"),
            Self::Iris => write!(f, "Iris"),
            Self::IrisStrobe => write!(f, "IrisStrobe"),
            Self::IrisStrobeRandom => write!(f, "IrisStrobeRandom"),
            Self::IrisPulseClose => write!(f, "IrisPulseClose"),
            Self::IrisPulseOpen => write!(f, "IrisPulseOpen"),
            Self::IrisRandomPulseClose => write!(f, "IrisRandomPulseClose"),
            Self::IrisRandomPulseOpen => write!(f, "IrisRandomPulseOpen"),
            Self::Frost(n) => write!(f, "Frost{n}"),
            Self::FrostPulseOpen(n) => write!(f, "Frost{n}PulseOpen"),
            Self::FrostPulseClose(n) => write!(f, "Frost{n}PulseClose"),
            Self::FrostRamp(n) => write!(f, "Frost{n}Ramp"),
            Self::Prism(n) => write!(f, "Prism{n}"),
            Self::PrismSelectSpin(n) => write!(f, "Prism{n}SelectSpin"),
            Self::PrismMacro(n) => write!(f, "Prism{n}Macro"),
            Self::PrismPos(n) => write!(f, "Prism{n}Pos"),
            Self::PrismPosRotate(n) => write!(f, "Prism{n}PosRotate"),

            Self::Effects(n) => write!(f, "Effects{n}"),
            Self::EffectsRate(n) => write!(f, "Effects{n}Rate"),
            Self::EffectsFade(n) => write!(f, "Effects{n}Fade"),
            Self::EffectsAdjust(n, m) => write!(f, "Effects{n}Adjust{m}"),
            Self::EffectsPos(n) => write!(f, "Effects{n}Pos"),
            Self::EffectsPosRotate(n) => write!(f, "Effects{n}PosRotate"),
            Self::EffectsSync => write!(f, "EffectsSync"),
            Self::BeamShaper => write!(f, "BeamShaper"),
            Self::BeamShaperMacro => write!(f, "BeamShaperMacro"),
            Self::BeamShaperPos => write!(f, "BeamShaperPos"),
            Self::BeamShaperPosRotate => write!(f, "BeamShaperPosRotate"),
            Self::Zoom => write!(f, "Zoom"),
            Self::ZoomModeSpot => write!(f, "ZoomModeSpot"),
            Self::ZoomModeBeam => write!(f, "ZoomModeBeam"),
            Self::DigitalZoom => write!(f, "DigitalZoom"),
            Self::Focus(n) => write!(f, "Focus{n}"),
            Self::FocusAdjust(n) => write!(f, "Focus{n}Adjust"),
            Self::FocusDistance(n) => write!(f, "Focus{n}Distance"),

            Self::Control(n) => write!(f, "Control{n}"),
            Self::DimmerMode => write!(f, "DimmerMode"),
            Self::DimmerCurve => write!(f, "DimmerCurve"),
            Self::BlackoutMode => write!(f, "BlackoutMode"),
            Self::LedFrequency => write!(f, "LedFrequency"),
            Self::LedZoneMode => write!(f, "LedZoneMode"),
            Self::PixelMode => write!(f, "PixelMode"),
            Self::PanMode => write!(f, "PanMode"),
            Self::TiltMode => write!(f, "TiltMode"),
            Self::PanTiltMode => write!(f, "PanTiltMode"),
            Self::PositionModes => write!(f, "PositionModes"),
            Self::GoboWheelMode(n) => write!(f, "Gobo{n}WheelMode"),
            Self::GoboWheelShortcutMode => write!(f, "GoboWheelShortcutMode"),
            Self::AnimationWheelMode(n) => write!(f, "Animation{n}WheelMode"),
            Self::AnimationWheelShortcutMode => write!(f, "AnimationWheelShortcutMode"),
            Self::ColorMode(n) => write!(f, "Color{n}Mode"),
            Self::ColorWheelShortcutMode => write!(f, "ColorWheelShortcutMode"),
            Self::CyanMode => write!(f, "CyanMode"),
            Self::MagentaMode => write!(f, "MagentaMode"),
            Self::YellowMode => write!(f, "YellowMode"),
            Self::ColorMixMode => write!(f, "ColorMixMode"),
            Self::ChromaticMode => write!(f, "ChromaticMode"),
            Self::ColorCalibrationMode => write!(f, "ColorCalibrationMode"),
            Self::ColorConsistency => write!(f, "ColorConsistency"),
            Self::ColorControl => write!(f, "ColorControl"),
            Self::ColorModelMode => write!(f, "ColorModelMode"),
            Self::ColorSettingsReset => write!(f, "ColorSettingsReset"),
            Self::ColorUniformity => write!(f, "ColorUniformity"),
            Self::CriMode => write!(f, "CriMode"),
            Self::CustomColor => write!(f, "CustomColor"),
            Self::UvStability => write!(f, "UvStability"),
            Self::WavelengthCorrection => write!(f, "WavelengthCorrection"),
            Self::WhiteCount => write!(f, "WhiteCount"),
            Self::StrobeMode => write!(f, "StrobeMode"),
            Self::ZoomMode => write!(f, "ZoomMode"),
            Self::FocusMode => write!(f, "FocusMode"),
            Self::IrisMode => write!(f, "IrisMode"),
            Self::FanMode(n) => write!(f, "Fan{n}Mode"),
            Self::FollowSpotMode => write!(f, "FollowSpotMode"),
            Self::BeamEffectIndexRotateMode => write!(f, "BeamEffectIndexRotateMode"),
            Self::IntensityMSpeed => write!(f, "IntensityMSpeed"),
            Self::PositionMSpeed => write!(f, "PositionMSpeed"),
            Self::ColorMixMSpeed => write!(f, "ColorMixMSpeed"),
            Self::ColorWheelSelectMSpeed => write!(f, "ColorWheelSelectMSpeed"),
            Self::GoboWheelMSpeed(n) => write!(f, "Gobo{n}WheelMSpeed"),
            Self::IrisMSpeed => write!(f, "IrisMSpeed"),
            Self::PrismMSpeed(n) => write!(f, "Prism{n}MSpeed"),
            Self::FocusMSpeed => write!(f, "FocusMSpeed"),
            Self::FrostMSpeed(n) => write!(f, "Frost{n}MSpeed"),
            Self::ZoomMSpeed => write!(f, "ZoomMSpeed"),
            Self::FrameMSpeed => write!(f, "FrameMSpeed"),
            Self::GlobalMSpeed => write!(f, "GlobalMSpeed"),
            Self::ReflectorAdjust => write!(f, "ReflectorAdjust"),
            Self::FixtureGlobalReset => write!(f, "FixtureGlobalReset"),
            Self::DimmerReset => write!(f, "DimmerReset"),
            Self::ShutterReset => write!(f, "ShutterReset"),
            Self::BeamReset => write!(f, "BeamReset"),
            Self::ColorMixReset => write!(f, "ColorMixReset"),
            Self::ColorWheelReset => write!(f, "ColorWheelReset"),
            Self::FocusReset => write!(f, "FocusReset"),
            Self::FrameReset => write!(f, "FrameReset"),
            Self::GoboWheelReset => write!(f, "GoboWheelReset"),
            Self::IntensityReset => write!(f, "IntensityReset"),
            Self::IrisReset => write!(f, "IrisReset"),
            Self::PositionReset => write!(f, "PositionReset"),
            Self::PanReset => write!(f, "PanReset"),
            Self::TiltReset => write!(f, "TiltReset"),
            Self::ZoomReset => write!(f, "ZoomReset"),
            Self::CtbReset => write!(f, "CTBReset"),
            Self::CtoReset => write!(f, "CTOReset"),
            Self::CtcReset => write!(f, "CTCReset"),
            Self::AnimationSystemReset => write!(f, "AnimationSystemReset"),
            Self::FixtureCalibrationReset => write!(f, "FixtureCalibrationReset"),
            Self::Function => write!(f, "Function"),
            Self::LampControl => write!(f, "LampControl"),
            Self::DisplayIntensity => write!(f, "DisplayIntensity"),
            Self::DmxInput => write!(f, "DMXInput"),
            Self::NoFeature => write!(f, "NoFeature"),
            Self::Blower(n) => write!(f, "Blower{n}"),
            Self::Fan(n) => write!(f, "Fan{n}"),
            Self::Fog(n) => write!(f, "Fog{n}"),
            Self::Haze(n) => write!(f, "Haze{n}"),
            Self::LampPowerMode => write!(f, "LampPowerMode"),
            Self::Fans => write!(f, "Fans"),

            Self::BladeA(n) => write!(f, "Blade{n}A"),
            Self::BladeB(n) => write!(f, "Blade{n}B"),
            Self::BladeRot(n) => write!(f, "Blade{n}Rot"),
            Self::ShaperRot => write!(f, "ShaperRot"),
            Self::ShaperMacros => write!(f, "ShaperMacros"),
            Self::ShaperMacrosSpeed => write!(f, "ShaperMacrosSpeed"),
            Self::BladeSoftA(n) => write!(f, "BladeSoft{n}A"),
            Self::BladeSoftB(n) => write!(f, "BladeSoft{n}B"),
            Self::KeyStoneA(n) => write!(f, "KeyStone{n}A"),
            Self::KeyStoneB(n) => write!(f, "KeyStone{n}B"),

            Self::Video => write!(f, "Video"),
            Self::VideoEffectType(n) => write!(f, "VideoEffect{n}Type"),
            Self::VideoEffectParameter(n, m) => write!(f, "VideoEffect{n}Parameter{m}"),
            Self::VideoCamera(n) => write!(f, "VideoCamera{n}"),
            Self::VideoSoundVolume(n) => write!(f, "VideoSoundVolume{n}"),
            Self::VideoBlendMode => write!(f, "VideoBlendMode"),
            Self::InputSource => write!(f, "InputSource"),
            Self::FieldOfView => write!(f, "FieldOfView"),

            Self::Custom(name) => write!(f, "{}", name.to_string()),
        }
    }
}

impl FromStr for FixtureChannel3Attribute {
    type Err = ();

    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Helper function to extract `n` from an attribute name.
        fn extract_attr_n(s: &str, prefix: &str, suffix: Option<&str>) -> Option<u8> {
            s.strip_prefix(prefix).and_then(|rest| {
                if let Some(suffix) = suffix {
                    rest.strip_suffix(suffix).and_then(|number_part| {
                        number_part.parse::<u8>().ok()
                    })
                } else {
                    rest.parse::<u8>().ok()
                }
            })
        }

        // Helper function to extract `n` and `m` from an attribute name.
        fn extract_attr_n_m(s: &str, prefix: &str, middle: &str, suffix: Option<&str>) -> Option<(u8, u8)> {
            // Check if it starts with the prefix
            if !s.starts_with(prefix) {
                return None;
            }

            let rest = &s[prefix.len()..];

            // Find the middle delimiter in the remaining string
            if let Some(middle_pos) = rest.find(middle) {
                // Extract the first number (n) before the middle delimiter
                let n_part = &rest[..middle_pos];
                let n = n_part.parse::<u8>().ok()?;

                // Get the part after the middle delimiter
                let after_middle = &rest[middle_pos + middle.len()..];

                // Handle optional suffix
                let m_part = if let Some(suffix) = suffix {
                    after_middle.strip_suffix(suffix)?
                } else {
                    after_middle
                };

                // Extract the second number (m)
                let m = m_part.parse::<u8>().ok()?;

                Some((n, m))
            } else {
                None
            }
        }

        let attribute = match s {
            "Dimmer" => Self::Dimmer,

            "Pan" => Self::Pan,
            "Tilt" => Self::Tilt,
            "PanRotate" => Self::PanRotate,
            "TiltRotate" => Self::TiltRotate,
            "PositionEffect" => Self::PositionEffect,
            "PositionEffectRate" => Self::PositionEffectRate,
            "PositionEffectFade" => Self::PositionEffectFade,
            "XYZ_X" => Self::XyzX,
            "XYZ_Y" => Self::XyzY,
            "XYZ_Z" => Self::XyzZ,
            "Rot_X" => Self::RotX,
            "Rot_Y" => Self::RotY,
            "Rot_Z" => Self::RotZ,
            "Scale_X" => Self::ScaleX,
            "Scale_Y" => Self::ScaleY,
            "Scale_Z" => Self::ScaleZ,
            "Scale_XYZ" => Self::ScaleXYZ,

            "PlayMode" => Self::PlayMode,
            "PlayBegin" => Self::PlayBegin,
            "PlayEnd" => Self::PlayEnd,
            "PlaySpeed" => Self::PlaySpeed,

            "ColorAdd_R" => Self::ColorAddR,
            "ColorAdd_G" => Self::ColorAddG,
            "ColorAdd_B" => Self::ColorAddB,
            "ColorAdd_C" => Self::ColorAddC,
            "ColorAdd_M" => Self::ColorAddM,
            "ColorAdd_Y" => Self::ColorAddY,
            "ColorAdd_RY" => Self::ColorAddRY,
            "ColorAdd_GY" => Self::ColorAddGY,
            "ColorAdd_GC" => Self::ColorAddGC,
            "ColorAdd_BC" => Self::ColorAddBC,
            "ColorAdd_BM" => Self::ColorAddBM,
            "ColorAdd_RM" => Self::ColorAddRM,
            "ColorAdd_W" => Self::ColorAddW,
            "ColorAdd_WW" => Self::ColorAddWW,
            "ColorAdd_CW" => Self::ColorAddCW,
            "ColorAdd_UV" => Self::ColorAddUV,
            "ColorSub_R" => Self::ColorSubR,
            "ColorSub_G" => Self::ColorSubG,
            "ColorSub_B" => Self::ColorSubB,
            "ColorSub_C" => Self::ColorSubC,
            "ColorSub_M" => Self::ColorSubM,
            "ColorSub_Y" => Self::ColorSubY,
            "CTO" => Self::Cto,
            "CTC" => Self::Ctc,
            "CTB" => Self::Ctb,
            "Tint" => Self::Tint,
            "HSB_Hue" => Self::HsbHue,
            "HSB_Saturation" => Self::HsbSaturation,
            "HSB_Brightness" => Self::HsbBrightness,
            "HSB_Quality" => Self::HsbQuality,
            "CIE_X" => Self::CieX,
            "CIE_Y" => Self::CieY,
            "CIE_Brightness" => Self::CieBrightness,
            "ColorRGB_Red" => Self::ColorRgbRed,
            "ColorRGB_Green" => Self::ColorRgbGreen,
            "ColorRGB_Blue" => Self::ColorRgbBlue,
            "ColorRGB_Cyan" => Self::ColorRgbCyan,
            "ColorRGB_Magenta" => Self::ColorRgbMagenta,
            "ColorRGB_Yellow" => Self::ColorRgbYellow,
            "ColorRGB_Quality" => Self::ColorRgbQuality,
            "VideoBoost_R" => Self::VideoBoostR,
            "VideoBoost_G" => Self::VideoBoostG,
            "VideoBoost_B" => Self::VideoBoostB,
            "VideoHueShift" => Self::VideoHueShift,
            "VideoSaturation" => Self::VideoSaturation,
            "VideoBrightness" => Self::VideoBrightness,
            "VideoContrast" => Self::VideoContrast,
            "VideoKeyColor_R" => Self::VideoKeyColorR,
            "VideoKeyColor_G" => Self::VideoKeyColorG,
            "VideoKeyColor_B" => Self::VideoKeyColorB,
            "VideoKeyIntensity" => Self::VideoKeyIntensity,
            "VideoKeyTolerance" => Self::VideoKeyTolerance,

            "StrobeDuration" => Self::StrobeDuration,
            "StrobeRate" => Self::StrobeRate,
            "StrobeFrequency" => Self::StrobeFrequency,
            "StrobeModeShutter" => Self::StrobeModeShutter,
            "StrobeModeStrobe" => Self::StrobeModeStrobe,
            "StrobeModePulse" => Self::StrobeModePulse,
            "StrobeModePulseOpen" => Self::StrobeModePulseOpen,
            "StrobeModePulseClose" => Self::StrobeModePulseClose,
            "StrobeModeRandom" => Self::StrobeModeRandom,
            "StrobeModeRandomPulse" => Self::StrobeModeRandomPulse,
            "StrobeModeRandomPulseOpen" => Self::StrobeModeRandomPulseOpen,
            "StrobeModeRandomPulseClose" => Self::StrobeModeRandomPulseClose,
            "StrobeModeEffect" => Self::StrobeModeEffect,
            "Iris" => Self::Iris,
            "IrisStrobe" => Self::IrisStrobe,
            "IrisStrobeRandom" => Self::IrisStrobeRandom,
            "IrisPulseClose" => Self::IrisPulseClose,
            "IrisPulseOpen" => Self::IrisPulseOpen,
            "IrisRandomPulseClose" => Self::IrisRandomPulseClose,
            "IrisRandomPulseOpen" => Self::IrisRandomPulseOpen,

            "EffectsSync" => Self::EffectsSync,
            "BeamShaper" => Self::BeamShaper,
            "BeamShaperMacro" => Self::BeamShaperMacro,
            "BeamShaperPos" => Self::BeamShaperPos,
            "BeamShaperPosRotate" => Self::BeamShaperPosRotate,
            "Zoom" => Self::Zoom,
            "ZoomModeSpot" => Self::ZoomModeSpot,
            "ZoomModeBeam" => Self::ZoomModeBeam,
            "DigitalZoom" => Self::DigitalZoom,

            "DimmerMode" => Self::DimmerMode,
            "DimmerCurve" => Self::DimmerCurve,
            "BlackoutMode" => Self::BlackoutMode,
            "LEDFrequency" => Self::LedFrequency,
            "LEDZoneMode" => Self::LedZoneMode,
            "PixelMode" => Self::PixelMode,
            "PanMode" => Self::PanMode,
            "TiltMode" => Self::TiltMode,
            "PanTiltMode" => Self::PanTiltMode,
            "PositionModes" => Self::PositionModes,
            "GoboWheelShortcutMode" => Self::GoboWheelShortcutMode,
            "AnimationWheelShortcutMode" => Self::AnimationWheelShortcutMode,
            "ColorWheelShortcutMode" => Self::ColorWheelShortcutMode,
            "CyanMode" => Self::CyanMode,
            "MagentaMode" => Self::MagentaMode,
            "YellowMode" => Self::YellowMode,
            "ColorMixMode" => Self::ColorMixMode,
            "ChromaticMode" => Self::ChromaticMode,
            "ColorCalibrationMode" => Self::ColorCalibrationMode,
            "ColorConsistency" => Self::ColorConsistency,
            "ColorControl" => Self::ColorControl,
            "ColorModelMode" => Self::ColorModelMode,
            "ColorSettingsReset" => Self::ColorSettingsReset,
            "ColorUniformity" => Self::ColorUniformity,
            "CRIMode" => Self::CriMode,
            "CustomColor" => Self::CustomColor,
            "UVStability" => Self::UvStability,
            "WavelengthCorrection" => Self::WavelengthCorrection,
            "WhiteCount" => Self::WhiteCount,
            "StrobeMode" => Self::StrobeMode,
            "ZoomMode" => Self::ZoomMode,
            "FocusMode" => Self::FocusMode,
            "IrisMode" => Self::IrisMode,
            "FollowSpotMode" => Self::FollowSpotMode,
            "BeamEffectIndexRotateMode" => Self::BeamEffectIndexRotateMode,
            "IntensityMSpeed" => Self::IntensityMSpeed,
            "PositionMSpeed" => Self::PositionMSpeed,
            "ColorMixMSpeed" => Self::ColorMixMSpeed,
            "ColorWheelSelectMSpeed" => Self::ColorWheelSelectMSpeed,
            "IrisMSpeed" => Self::IrisMSpeed,
            "FocusMSpeed" => Self::FocusMSpeed,
            "ZoomMSpeed" => Self::ZoomMSpeed,
            "FrameMSpeed" => Self::FrameMSpeed,
            "GlobalMSpeed" => Self::GlobalMSpeed,
            "ReflectorAdjust" => Self::ReflectorAdjust,
            "FixtureGlobalReset" => Self::FixtureGlobalReset,
            "DimmerReset" => Self::DimmerReset,
            "ShutterReset" => Self::ShutterReset,
            "BeamReset" => Self::BeamReset,
            "ColorMixReset" => Self::ColorMixReset,
            "ColorWheelReset" => Self::ColorWheelReset,
            "FocusReset" => Self::FocusReset,
            "FrameReset" => Self::FrameReset,
            "GoboWheelReset" => Self::GoboWheelReset,
            "IntensityReset" => Self::IntensityReset,
            "IrisReset" => Self::IrisReset,
            "PositionReset" => Self::PositionReset,
            "PanReset" => Self::PanReset,
            "TiltReset" => Self::TiltReset,
            "ZoomReset" => Self::ZoomReset,
            "CTBReset" => Self::CtbReset,
            "CTOReset" => Self::CtoReset,
            "CTCReset" => Self::CtcReset,
            "AnimationSystemReset" => Self::AnimationSystemReset,
            "FixtureCalibrationReset" => Self::FixtureCalibrationReset,
            "Function" => Self::Function,
            "LampControl" => Self::LampControl,
            "DisplayIntensity" => Self::DisplayIntensity,
            "DMXInput" => Self::DmxInput,
            "NoFeature" => Self::NoFeature,

            "LampPowerMode" => Self::LampPowerMode,
            "Fans" => Self::Fans,

            "ShaperRot" => Self::ShaperRot,
            "ShaperMacros" => Self::ShaperMacros,
            "ShaperMacrosSpeed" => Self::ShaperMacrosSpeed,

            "Video" => Self::Video,
            "VideoBlendMode" => Self::VideoBlendMode,
            "InputSource" => Self::InputSource,
            "FieldOfView" => Self::FieldOfView,

            s => {
                     if let Some(n) = extract_attr_n(s, "Gobo", None) { Self::Gobo(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("SelectSpin")) { Self::GoboSelectSpin(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("SelectShake")) { Self::GoboSelectShake(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("SelectEffects")) { Self::GoboSelectEffects(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("WheelIndex")) { Self::GoboWheelIndex(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("WheelSpin")) { Self::GoboWheelSpin(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("WheelShake")) { Self::GoboWheelShake(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("WheelRandom")) { Self::GoboWheelRandom(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("WheelAudio")) { Self::GoboWheelAudio(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("Pos")) { Self::GoboPos(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("PosRotate")) { Self::GoboPosRotate(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("PosShake")) { Self::GoboPosShake(n) }

                else if let Some(n) = extract_attr_n(s, "AnimationWheel", None) { Self::AnimationWheel(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("Audio")) { Self::AnimationWheelAudio(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("Macro")) { Self::AnimationWheelMacro(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("Random")) { Self::AnimationWheelRandom(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("SelectEffects")) { Self::AnimationWheelSelectEffects(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("SelectShake")) { Self::AnimationWheelSelectShake(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("SelectSpin")) { Self::AnimationWheelSelectSpin(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("Pos")) { Self::AnimationWheelPos(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("PosRotate")) { Self::AnimationWheelPosRotate(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("PosShake")) { Self::AnimationWheelPosShake(n) }

                else if let Some(n) = extract_attr_n(s, "AnimationSystem", None) { Self::AnimationSystem(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("Ramp")) { Self::AnimationSystemRamp(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("Shake")) { Self::AnimationSystemShake(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("Audio")) { Self::AnimationSystemAudio(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("Random")) { Self::AnimationSystemRandom(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("Pos")) { Self::AnimationSystemPos(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("PosRotate")) { Self::AnimationSystemPosRotate(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("PosShake")) { Self::AnimationSystemPosShake(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("PosRandom")) { Self::AnimationSystemPosRandom(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("PosAudio")) { Self::AnimationSystemPosAudio(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationSystem", Some("Macro")) { Self::AnimationSystemMacro(n) }

                else if let Some(n) = extract_attr_n(s, "MediaFolder",  None) { Self::MediaFolder(n) }
                else if let Some(n) = extract_attr_n(s, "MediaContent", None) { Self::MediaContent(n) }
                else if let Some(n) = extract_attr_n(s, "ModelFolder",  None) { Self::ModelFolder(n) }
                else if let Some(n) = extract_attr_n(s, "ModelContent", None) { Self::ModelContent(n) }

                else if let Some(n) = extract_attr_n(s, "ColorEffects", None) { Self::ColorEffects(n) }
                else if let Some(n) = extract_attr_n(s, "Color", None) { Self::Color(n) }
                else if let Some(n) = extract_attr_n(s, "Color", Some("WheelIndex")) { Self::ColorWheelIndex(n) }
                else if let Some(n) = extract_attr_n(s, "Color", Some("WheelSpin")) { Self::ColorWheelSpin(n) }
                else if let Some(n) = extract_attr_n(s, "Color", Some("WheelRandom")) { Self::ColorWheelRandom(n) }
                else if let Some(n) = extract_attr_n(s, "Color", Some("WheelAudio")) { Self::ColorWheelAudio(n) }

                else if let Some(n) = extract_attr_n(s, "ColorMacro", None) { Self::ColorMacro(n) }
                else if let Some(n) = extract_attr_n(s, "ColorMacro", Some("Rate")) { Self::ColorMacroRate(n) }

                else if let Some(n) = extract_attr_n(s, "Shutter", None) { Self::Shutter(n) }
                else if let Some(n) = extract_attr_n(s, "Shutter", Some("Strobe")) { Self::ShutterStrobe(n) }
                else if let Some(n) = extract_attr_n(s, "Shutter", Some("StrobePulse")) { Self::ShutterStrobePulse(n) }
                else if let Some(n) = extract_attr_n(s, "Shutter", Some("StrobePulseClose")) { Self::ShutterStrobePulseClose(n) }
                else if let Some(n) = extract_attr_n(s, "Shutter", Some("StrobePulseOpen")) { Self::ShutterStrobePulseOpen(n) }
                else if let Some(n) = extract_attr_n(s, "Shutter", Some("StrobeRandom")) { Self::ShutterStrobeRandom(n) }
                else if let Some(n) = extract_attr_n(s, "Shutter", Some("StrobeRandomPulse")) { Self::ShutterStrobeRandomPulse(n) }
                else if let Some(n) = extract_attr_n(s, "Shutter", Some("StrobeRandomPulseClose")) { Self::ShutterStrobeRandomPulseClose(n) }
                else if let Some(n) = extract_attr_n(s, "Shutter", Some("StrobeRandomPulseOpen")) { Self::ShutterStrobeRandomPulseOpen(n) }
                else if let Some(n) = extract_attr_n(s, "Shutter", Some("StrobeEffect")) { Self::ShutterStrobeEffect(n) }

                else if let Some(n) = extract_attr_n(s, "Frost", None) { Self::Frost(n) }
                else if let Some(n) = extract_attr_n(s, "Frost", Some("PulseOpen")) { Self::FrostPulseOpen(n) }
                else if let Some(n) = extract_attr_n(s, "Frost", Some("PulseClose")) { Self::FrostPulseClose(n) }
                else if let Some(n) = extract_attr_n(s, "Frost", Some("Ramp")) { Self::FrostRamp(n) }

                else if let Some(n) = extract_attr_n(s, "Prism", None) { Self::Prism(n) }
                else if let Some(n) = extract_attr_n(s, "Prism", Some("SelectSpin")) { Self::PrismSelectSpin(n) }
                else if let Some(n) = extract_attr_n(s, "Prism", Some("Macro")) { Self::PrismMacro(n) }
                else if let Some(n) = extract_attr_n(s, "Prism", Some("Pos")) { Self::PrismPos(n) }
                else if let Some(n) = extract_attr_n(s, "Prism", Some("PosRotate")) { Self::PrismPosRotate(n) }

                else if let Some((n, m)) = extract_attr_n_m(s, "Effects", "Adjust", None) { Self::EffectsAdjust(n, m) }
                else if let Some(n) = extract_attr_n(s, "Effects", None) { Self::Effects(n) }
                else if let Some(n) = extract_attr_n(s, "Effects", Some("Rate")) { Self::EffectsRate(n) }
                else if let Some(n) = extract_attr_n(s, "Effects", Some("Fade")) { Self::EffectsFade(n) }
                else if let Some(n) = extract_attr_n(s, "Effects", Some("Pos")) { Self::EffectsPos(n) }
                else if let Some(n) = extract_attr_n(s, "Effects", Some("PosRotate")) { Self::EffectsPosRotate(n) }
                else if let Some(n) = extract_attr_n(s, "Focus", None) { Self::Focus(n) }
                else if let Some(n) = extract_attr_n(s, "Focus", Some("Adjust")) { Self::FocusAdjust(n) }
                else if let Some(n) = extract_attr_n(s, "Focus", Some("Distance")) { Self::FocusDistance(n) }
                else if let Some(n) = extract_attr_n(s, "Control", None) { Self::Control(n) }
                else if let Some(n) = extract_attr_n(s, "Gobo", Some("WheelMode")) { Self::GoboWheelMode(n) }
                else if let Some(n) = extract_attr_n(s, "AnimationWheel", Some("Mode")) { Self::AnimationWheelMode(n) }
                else if let Some(n) = extract_attr_n(s, "Color", Some("Mode")) { Self::ColorMode(n) }

                else if let Some(n) = extract_attr_n(s, "Fan", Some("Mode")) { Self::FanMode(n) }
                else if let Some(n) = extract_attr_n(s, "GoboWheel", Some("MSpeed")) { Self::GoboWheelMSpeed(n) }
                else if let Some(n) = extract_attr_n(s, "Prism", Some("MSpeed")) { Self::PrismMSpeed(n) }
                else if let Some(n) = extract_attr_n(s, "Frost", Some("MSpeed")) { Self::FrostMSpeed(n) }
                else if let Some(n) = extract_attr_n(s, "Blower", None) { Self::Blower(n) }
                else if let Some(n) = extract_attr_n(s, "Fan", None) { Self::Fan(n) }
                else if let Some(n) = extract_attr_n(s, "Fog", None) { Self::Fog(n) }
                else if let Some(n) = extract_attr_n(s, "Haze", None) { Self::Haze(n) }

                else if let Some(n) = extract_attr_n(s, "Blade", Some("A")) { Self::BladeA(n) }
                else if let Some(n) = extract_attr_n(s, "Blade", Some("B")) { Self::BladeB(n) }
                else if let Some(n) = extract_attr_n(s, "Blade", Some("Rot")) { Self::BladeRot(n) }
                else if let Some(n) = extract_attr_n(s, "BladeSoft", Some("A")) { Self::BladeSoftA(n) }
                else if let Some(n) = extract_attr_n(s, "BladeSoft", Some("B")) { Self::BladeSoftB(n) }
                else if let Some(n) = extract_attr_n(s, "KeyStone", Some("A")) { Self::KeyStoneA(n) }
                else if let Some(n) = extract_attr_n(s, "KeyStone", Some("B")) { Self::KeyStoneB(n) }

                else if let Some(n) = extract_attr_n(s, "VideoEffect", Some("Type")) { Self::VideoEffectType(n) }
                else if let Some((n, m)) = extract_attr_n_m(s, "VideoEffect", "Parameter", None) { Self::VideoEffectParameter(n, m) }
                else if let Some(n) = extract_attr_n(s, "VideoCamera", None) { Self::VideoCamera(n) }
                else if let Some(n) = extract_attr_n(s, "VideoSoundVolume", None) { Self::VideoSoundVolume(n) }

                else { Self::Custom(CustomName::new(s.to_string())) }
            }
        };

        Ok(attribute)
    }
}

impl serde::Serialize for FixtureChannel3Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for FixtureChannel3Attribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct AttributeVisitor;

        impl<'de> Visitor<'de> for AttributeVisitor {
            type Value = FixtureChannel3Attribute;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing an Attribute")
            }

            fn visit_str<E>(self, v: &str) -> Result<FixtureChannel3Attribute, E>
            where
                E: de::Error,
            {
                FixtureChannel3Attribute::from_str(v)
                    .map_err(|_| E::custom(format!("invalid Attribute string: {v}")))
            }
        }

        deserializer.deserialize_str(AttributeVisitor)
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use std::str::FromStr;

    use super::*;

    macro_rules! test_attr {
        ($test_name:ident, $attr_name:literal, $attr_kind:expr) => {
            #[test]
            fn $test_name() {
                assert_eq!(super::FixtureChannel3Attribute::from_str($attr_name).unwrap(), $attr_kind);
            }
        };
    }

    test_attr!(dimmer, "Dimmer", FixtureChannel3Attribute::Dimmer);
    test_attr!(pan, "Pan", FixtureChannel3Attribute::Pan);
    test_attr!(tilt, "Tilt", FixtureChannel3Attribute::Tilt);
    test_attr!(pan_rotate, "PanRotate", FixtureChannel3Attribute::PanRotate);
    test_attr!(tilt_rotate, "TiltRotate", FixtureChannel3Attribute::TiltRotate);
    test_attr!(position_effect, "PositionEffect", FixtureChannel3Attribute::PositionEffect);
    test_attr!(position_effect_rate, "PositionEffectRate", FixtureChannel3Attribute::PositionEffectRate);
    test_attr!(position_effect_fade, "PositionEffectFade", FixtureChannel3Attribute::PositionEffectFade);
    test_attr!(xyz_x, "XYZ_X", FixtureChannel3Attribute::XyzX);
    test_attr!(xyz_y, "XYZ_Y", FixtureChannel3Attribute::XyzY);
    test_attr!(xyz_z, "XYZ_Z", FixtureChannel3Attribute::XyzZ);
    test_attr!(rot_x, "Rot_X", FixtureChannel3Attribute::RotX);
    test_attr!(rot_y, "Rot_Y", FixtureChannel3Attribute::RotY);
    test_attr!(rot_z, "Rot_Z", FixtureChannel3Attribute::RotZ);
    test_attr!(scale_x, "Scale_X", FixtureChannel3Attribute::ScaleX);
    test_attr!(scale_y, "Scale_Y", FixtureChannel3Attribute::ScaleY);
    test_attr!(scale_z, "Scale_Z", FixtureChannel3Attribute::ScaleZ);
    test_attr!(scale_xyz, "Scale_XYZ", FixtureChannel3Attribute::ScaleXYZ);
    test_attr!(gobo_n, "Gobo1", FixtureChannel3Attribute::Gobo(1));
    test_attr!(gobo_n_select_spin, "Gobo1SelectSpin", FixtureChannel3Attribute::GoboSelectSpin(1));
    test_attr!(gobo_n_select_shake, "Gobo1SelectShake", FixtureChannel3Attribute::GoboSelectShake(1));
    test_attr!(gobo_n_select_effects, "Gobo1SelectEffects", FixtureChannel3Attribute::GoboSelectEffects(1));
    test_attr!(gobo_n_wheel_index, "Gobo1WheelIndex", FixtureChannel3Attribute::GoboWheelIndex(1));
    test_attr!(gobo_n_wheel_spin, "Gobo1WheelSpin", FixtureChannel3Attribute::GoboWheelSpin(1));
    test_attr!(gobo_n_wheel_shake, "Gobo1WheelShake", FixtureChannel3Attribute::GoboWheelShake(1));
    test_attr!(gobo_n_wheel_random, "Gobo1WheelRandom", FixtureChannel3Attribute::GoboWheelRandom(1));
    test_attr!(gobo_n_wheel_audio, "Gobo1WheelAudio", FixtureChannel3Attribute::GoboWheelAudio(1));
    test_attr!(gobo_n_pos, "Gobo1Pos", FixtureChannel3Attribute::GoboPos(1));
    test_attr!(gobo_n_pos_rotate, "Gobo1PosRotate", FixtureChannel3Attribute::GoboPosRotate(1));
    test_attr!(gobo_n_pos_shake, "Gobo1PosShake", FixtureChannel3Attribute::GoboPosShake(1));
    test_attr!(animation_wheel_n, "AnimationWheel1", FixtureChannel3Attribute::AnimationWheel(1));
    test_attr!(animation_wheel_n_audio, "AnimationWheel1Audio", FixtureChannel3Attribute::AnimationWheelAudio(1));
    test_attr!(animation_wheel_n_macro, "AnimationWheel1Macro", FixtureChannel3Attribute::AnimationWheelMacro(1));
    test_attr!(animation_wheel_n_random, "AnimationWheel1Random", FixtureChannel3Attribute::AnimationWheelRandom(1));
    test_attr!(animation_wheel_n_select_effects, "AnimationWheel1SelectEffects", FixtureChannel3Attribute::AnimationWheelSelectEffects(1));
    test_attr!(animation_wheel_n_select_shake, "AnimationWheel1SelectShake", FixtureChannel3Attribute::AnimationWheelSelectShake(1));
    test_attr!(animation_wheel_n_select_spin, "AnimationWheel1SelectSpin", FixtureChannel3Attribute::AnimationWheelSelectSpin(1));
    test_attr!(animation_wheel_n_pos, "AnimationWheel1Pos", FixtureChannel3Attribute::AnimationWheelPos(1));
    test_attr!(animation_wheel_n_pos_rotate, "AnimationWheel1PosRotate", FixtureChannel3Attribute::AnimationWheelPosRotate(1));
    test_attr!(animation_wheel_n_pos_shake, "AnimationWheel1PosShake", FixtureChannel3Attribute::AnimationWheelPosShake(1));
    test_attr!(animation_system_n, "AnimationSystem1", FixtureChannel3Attribute::AnimationSystem(1));
    test_attr!(animation_system_n_ramp, "AnimationSystem1Ramp", FixtureChannel3Attribute::AnimationSystemRamp(1));
    test_attr!(animation_system_n_shake, "AnimationSystem1Shake", FixtureChannel3Attribute::AnimationSystemShake(1));
    test_attr!(animation_system_n_audio, "AnimationSystem1Audio", FixtureChannel3Attribute::AnimationSystemAudio(1));
    test_attr!(animation_system_n_random, "AnimationSystem1Random", FixtureChannel3Attribute::AnimationSystemRandom(1));
    test_attr!(animation_system_n_pos, "AnimationSystem1Pos", FixtureChannel3Attribute::AnimationSystemPos(1));
    test_attr!(animation_system_n_pos_rotate, "AnimationSystem1PosRotate", FixtureChannel3Attribute::AnimationSystemPosRotate(1));
    test_attr!(animation_system_n_pos_shake, "AnimationSystem1PosShake", FixtureChannel3Attribute::AnimationSystemPosShake(1));
    test_attr!(animation_system_n_pos_random, "AnimationSystem1PosRandom", FixtureChannel3Attribute::AnimationSystemPosRandom(1));
    test_attr!(animation_system_n_pos_audio, "AnimationSystem1PosAudio", FixtureChannel3Attribute::AnimationSystemPosAudio(1));
    test_attr!(animation_system_n_macro, "AnimationSystem1Macro", FixtureChannel3Attribute::AnimationSystemMacro(1));
    test_attr!(media_folder_n, "MediaFolder1", FixtureChannel3Attribute::MediaFolder(1));
    test_attr!(media_content_n, "MediaContent1", FixtureChannel3Attribute::MediaContent(1));
    test_attr!(model_folder_n, "ModelFolder1", FixtureChannel3Attribute::ModelFolder(1));
    test_attr!(model_content_n, "ModelContent1", FixtureChannel3Attribute::ModelContent(1));
    test_attr!(play_mode, "PlayMode", FixtureChannel3Attribute::PlayMode);
    test_attr!(play_begin, "PlayBegin", FixtureChannel3Attribute::PlayBegin);
    test_attr!(play_end, "PlayEnd", FixtureChannel3Attribute::PlayEnd);
    test_attr!(play_speed, "PlaySpeed", FixtureChannel3Attribute::PlaySpeed);
    test_attr!(color_effects_n, "ColorEffects1", FixtureChannel3Attribute::ColorEffects(1));
    test_attr!(color_n, "Color1", FixtureChannel3Attribute::Color(1));
    test_attr!(color_n_wheel_index, "Color1WheelIndex", FixtureChannel3Attribute::ColorWheelIndex(1));
    test_attr!(color_n_wheel_spin, "Color1WheelSpin", FixtureChannel3Attribute::ColorWheelSpin(1));
    test_attr!(color_n_wheel_random, "Color1WheelRandom", FixtureChannel3Attribute::ColorWheelRandom(1));
    test_attr!(color_n_wheel_audio, "Color1WheelAudio", FixtureChannel3Attribute::ColorWheelAudio(1));
    test_attr!(color_add_r, "ColorAdd_R", FixtureChannel3Attribute::ColorAddR);
    test_attr!(color_add_g, "ColorAdd_G", FixtureChannel3Attribute::ColorAddG);
    test_attr!(color_add_b, "ColorAdd_B", FixtureChannel3Attribute::ColorAddB);
    test_attr!(color_add_c, "ColorAdd_C", FixtureChannel3Attribute::ColorAddC);
    test_attr!(color_add_m, "ColorAdd_M", FixtureChannel3Attribute::ColorAddM);
    test_attr!(color_add_y, "ColorAdd_Y", FixtureChannel3Attribute::ColorAddY);
    test_attr!(color_add_ry, "ColorAdd_RY", FixtureChannel3Attribute::ColorAddRY);
    test_attr!(color_add_gy, "ColorAdd_GY", FixtureChannel3Attribute::ColorAddGY);
    test_attr!(color_add_gc, "ColorAdd_GC", FixtureChannel3Attribute::ColorAddGC);
    test_attr!(color_add_bc, "ColorAdd_BC", FixtureChannel3Attribute::ColorAddBC);
    test_attr!(color_add_bm, "ColorAdd_BM", FixtureChannel3Attribute::ColorAddBM);
    test_attr!(color_add_rm, "ColorAdd_RM", FixtureChannel3Attribute::ColorAddRM);
    test_attr!(color_add_w, "ColorAdd_W", FixtureChannel3Attribute::ColorAddW);
    test_attr!(color_add_ww, "ColorAdd_WW", FixtureChannel3Attribute::ColorAddWW);
    test_attr!(color_add_cw, "ColorAdd_CW", FixtureChannel3Attribute::ColorAddCW);
    test_attr!(color_add_uv, "ColorAdd_UV", FixtureChannel3Attribute::ColorAddUV);
    test_attr!(color_sub_r, "ColorSub_R", FixtureChannel3Attribute::ColorSubR);
    test_attr!(color_sub_g, "ColorSub_G", FixtureChannel3Attribute::ColorSubG);
    test_attr!(color_sub_b, "ColorSub_B", FixtureChannel3Attribute::ColorSubB);
    test_attr!(color_sub_c, "ColorSub_C", FixtureChannel3Attribute::ColorSubC);
    test_attr!(color_sub_m, "ColorSub_M", FixtureChannel3Attribute::ColorSubM);
    test_attr!(color_sub_y, "ColorSub_Y", FixtureChannel3Attribute::ColorSubY);
    test_attr!(color_macro_n, "ColorMacro1", FixtureChannel3Attribute::ColorMacro(1));
    test_attr!(color_macro_n_rate, "ColorMacro1Rate", FixtureChannel3Attribute::ColorMacroRate(1));
    test_attr!(cto, "CTO", FixtureChannel3Attribute::Cto);
    test_attr!(ctc, "CTC", FixtureChannel3Attribute::Ctc);
    test_attr!(ctb, "CTB", FixtureChannel3Attribute::Ctb);
    test_attr!(tint, "Tint", FixtureChannel3Attribute::Tint);
    test_attr!(hsb_hue, "HSB_Hue", FixtureChannel3Attribute::HsbHue);
    test_attr!(hsb_saturation, "HSB_Saturation", FixtureChannel3Attribute::HsbSaturation);
    test_attr!(hsb_brightness, "HSB_Brightness", FixtureChannel3Attribute::HsbBrightness);
    test_attr!(hsb_quality, "HSB_Quality", FixtureChannel3Attribute::HsbQuality);
    test_attr!(cie_x, "CIE_X", FixtureChannel3Attribute::CieX);
    test_attr!(cie_y, "CIE_Y", FixtureChannel3Attribute::CieY);
    test_attr!(cie_brightness, "CIE_Brightness", FixtureChannel3Attribute::CieBrightness);
    test_attr!(color_rgb_red, "ColorRGB_Red", FixtureChannel3Attribute::ColorRgbRed);
    test_attr!(color_rgb_green, "ColorRGB_Green", FixtureChannel3Attribute::ColorRgbGreen);
    test_attr!(color_rgb_blue, "ColorRGB_Blue", FixtureChannel3Attribute::ColorRgbBlue);
    test_attr!(color_rgb_cyan, "ColorRGB_Cyan", FixtureChannel3Attribute::ColorRgbCyan);
    test_attr!(color_rgb_magenta, "ColorRGB_Magenta", FixtureChannel3Attribute::ColorRgbMagenta);
    test_attr!(color_rgb_yellow, "ColorRGB_Yellow", FixtureChannel3Attribute::ColorRgbYellow);
    test_attr!(color_rgb_quality, "ColorRGB_Quality", FixtureChannel3Attribute::ColorRgbQuality);
    test_attr!(video_boost_r, "VideoBoost_R", FixtureChannel3Attribute::VideoBoostR);
    test_attr!(video_boost_g, "VideoBoost_G", FixtureChannel3Attribute::VideoBoostG);
    test_attr!(video_boost_b, "VideoBoost_B", FixtureChannel3Attribute::VideoBoostB);
    test_attr!(video_hue_shift, "VideoHueShift", FixtureChannel3Attribute::VideoHueShift);
    test_attr!(video_saturation, "VideoSaturation", FixtureChannel3Attribute::VideoSaturation);
    test_attr!(video_brightness, "VideoBrightness", FixtureChannel3Attribute::VideoBrightness);
    test_attr!(video_contrast, "VideoContrast", FixtureChannel3Attribute::VideoContrast);
    test_attr!(video_key_color_r, "VideoKeyColor_R", FixtureChannel3Attribute::VideoKeyColorR);
    test_attr!(video_key_color_g, "VideoKeyColor_G", FixtureChannel3Attribute::VideoKeyColorG);
    test_attr!(video_key_color_b, "VideoKeyColor_B", FixtureChannel3Attribute::VideoKeyColorB);
    test_attr!(video_key_intensity, "VideoKeyIntensity", FixtureChannel3Attribute::VideoKeyIntensity);
    test_attr!(video_key_tolerance, "VideoKeyTolerance", FixtureChannel3Attribute::VideoKeyTolerance);
    test_attr!(strobe_duration, "StrobeDuration", FixtureChannel3Attribute::StrobeDuration);
    test_attr!(strobe_rate, "StrobeRate", FixtureChannel3Attribute::StrobeRate);
    test_attr!(strobe_frequency, "StrobeFrequency", FixtureChannel3Attribute::StrobeFrequency);
    test_attr!(strobe_mode_shutter, "StrobeModeShutter", FixtureChannel3Attribute::StrobeModeShutter);
    test_attr!(strobe_mode_strobe, "StrobeModeStrobe", FixtureChannel3Attribute::StrobeModeStrobe);
    test_attr!(strobe_mode_pulse, "StrobeModePulse", FixtureChannel3Attribute::StrobeModePulse);
    test_attr!(strobe_mode_pulse_open, "StrobeModePulseOpen", FixtureChannel3Attribute::StrobeModePulseOpen);
    test_attr!(strobe_mode_pulse_close, "StrobeModePulseClose", FixtureChannel3Attribute::StrobeModePulseClose);
    test_attr!(strobe_mode_random, "StrobeModeRandom", FixtureChannel3Attribute::StrobeModeRandom);
    test_attr!(strobe_mode_random_pulse, "StrobeModeRandomPulse", FixtureChannel3Attribute::StrobeModeRandomPulse);
    test_attr!(strobe_mode_random_pulse_open, "StrobeModeRandomPulseOpen", FixtureChannel3Attribute::StrobeModeRandomPulseOpen);
    test_attr!(strobe_mode_random_pulse_close, "StrobeModeRandomPulseClose", FixtureChannel3Attribute::StrobeModeRandomPulseClose);
    test_attr!(strobe_mode_effect, "StrobeModeEffect", FixtureChannel3Attribute::StrobeModeEffect);
    test_attr!(shutter_n, "Shutter1", FixtureChannel3Attribute::Shutter(1));
    test_attr!(shutter_n_strobe, "Shutter1Strobe", FixtureChannel3Attribute::ShutterStrobe(1));
    test_attr!(shutter_n_strobe_pulse, "Shutter1StrobePulse", FixtureChannel3Attribute::ShutterStrobePulse(1));
    test_attr!(shutter_n_strobe_pulse_close, "Shutter1StrobePulseClose", FixtureChannel3Attribute::ShutterStrobePulseClose(1));
    test_attr!(shutter_n_strobe_pulse_open, "Shutter1StrobePulseOpen", FixtureChannel3Attribute::ShutterStrobePulseOpen(1));
    test_attr!(shutter_n_strobe_random, "Shutter1StrobeRandom", FixtureChannel3Attribute::ShutterStrobeRandom(1));
    test_attr!(shutter_n_strobe_random_pulse, "Shutter1StrobeRandomPulse", FixtureChannel3Attribute::ShutterStrobeRandomPulse(1));
    test_attr!(shutter_n_strobe_random_pulse_close, "Shutter1StrobeRandomPulseClose", FixtureChannel3Attribute::ShutterStrobeRandomPulseClose(1));
    test_attr!(shutter_n_strobe_random_pulse_open, "Shutter1StrobeRandomPulseOpen", FixtureChannel3Attribute::ShutterStrobeRandomPulseOpen(1));
    test_attr!(shutter_n_strobe_effect, "Shutter1StrobeEffect", FixtureChannel3Attribute::ShutterStrobeEffect(1));
    test_attr!(iris, "Iris", FixtureChannel3Attribute::Iris);
    test_attr!(iris_strobe, "IrisStrobe", FixtureChannel3Attribute::IrisStrobe);
    test_attr!(iris_strobe_random, "IrisStrobeRandom", FixtureChannel3Attribute::IrisStrobeRandom);
    test_attr!(iris_pulse_close, "IrisPulseClose", FixtureChannel3Attribute::IrisPulseClose);
    test_attr!(iris_pulse_open, "IrisPulseOpen", FixtureChannel3Attribute::IrisPulseOpen);
    test_attr!(iris_random_pulse_close, "IrisRandomPulseClose", FixtureChannel3Attribute::IrisRandomPulseClose);
    test_attr!(iris_random_pulse_open, "IrisRandomPulseOpen", FixtureChannel3Attribute::IrisRandomPulseOpen);
    test_attr!(frost_n, "Frost1", FixtureChannel3Attribute::Frost(1));
    test_attr!(frost_n_pulse_open, "Frost1PulseOpen", FixtureChannel3Attribute::FrostPulseOpen(1));
    test_attr!(frost_n_pulse_close, "Frost1PulseClose", FixtureChannel3Attribute::FrostPulseClose(1));
    test_attr!(frost_n_ramp, "Frost1Ramp", FixtureChannel3Attribute::FrostRamp(1));
    test_attr!(prism_n, "Prism1", FixtureChannel3Attribute::Prism(1));
    test_attr!(prism_n_select_spin, "Prism1SelectSpin", FixtureChannel3Attribute::PrismSelectSpin(1));
    test_attr!(prism_n_macro, "Prism1Macro", FixtureChannel3Attribute::PrismMacro(1));
    test_attr!(prism_n_pos, "Prism1Pos", FixtureChannel3Attribute::PrismPos(1));
    test_attr!(prism_n_pos_rotate, "Prism1PosRotate", FixtureChannel3Attribute::PrismPosRotate(1));
    test_attr!(effects_n, "Effects1", FixtureChannel3Attribute::Effects(1));
    test_attr!(effects_n_rate, "Effects1Rate", FixtureChannel3Attribute::EffectsRate(1));
    test_attr!(effects_n_fade, "Effects1Fade", FixtureChannel3Attribute::EffectsFade(1));
    test_attr!(effects_n_adjust_m, "Effects1Adjust2", FixtureChannel3Attribute::EffectsAdjust(1, 2));
    test_attr!(effects_n_pos, "Effects1Pos", FixtureChannel3Attribute::EffectsPos(1));
    test_attr!(effects_n_pos_rotate, "Effects1PosRotate", FixtureChannel3Attribute::EffectsPosRotate(1));
    test_attr!(effects_sync, "EffectsSync", FixtureChannel3Attribute::EffectsSync);
    test_attr!(beam_shaper, "BeamShaper", FixtureChannel3Attribute::BeamShaper);
    test_attr!(beam_shaper_macro, "BeamShaperMacro", FixtureChannel3Attribute::BeamShaperMacro);
    test_attr!(beam_shaper_pos, "BeamShaperPos", FixtureChannel3Attribute::BeamShaperPos);
    test_attr!(beam_shaper_pos_rotate, "BeamShaperPosRotate", FixtureChannel3Attribute::BeamShaperPosRotate);
    test_attr!(zoom, "Zoom", FixtureChannel3Attribute::Zoom);
    test_attr!(zoom_mode_spot, "ZoomModeSpot", FixtureChannel3Attribute::ZoomModeSpot);
    test_attr!(zoom_mode_beam, "ZoomModeBeam", FixtureChannel3Attribute::ZoomModeBeam);
    test_attr!(digital_zoom, "DigitalZoom", FixtureChannel3Attribute::DigitalZoom);
    test_attr!(focus_n, "Focus1", FixtureChannel3Attribute::Focus(1));
    test_attr!(focus_n_adjust, "Focus1Adjust", FixtureChannel3Attribute::FocusAdjust(1));
    test_attr!(focus_n_distance, "Focus1Distance", FixtureChannel3Attribute::FocusDistance(1));
    test_attr!(control_n, "Control1", FixtureChannel3Attribute::Control(1));
    test_attr!(dimmer_mode, "DimmerMode", FixtureChannel3Attribute::DimmerMode);
    test_attr!(dimmer_curve, "DimmerCurve", FixtureChannel3Attribute::DimmerCurve);
    test_attr!(blackout_mode, "BlackoutMode", FixtureChannel3Attribute::BlackoutMode);
    test_attr!(led_frequency, "LEDFrequency", FixtureChannel3Attribute::LedFrequency);
    test_attr!(led_zone_mode, "LEDZoneMode", FixtureChannel3Attribute::LedZoneMode);
    test_attr!(pixel_mode, "PixelMode", FixtureChannel3Attribute::PixelMode);
    test_attr!(pan_mode, "PanMode", FixtureChannel3Attribute::PanMode);
    test_attr!(tilt_mode, "TiltMode", FixtureChannel3Attribute::TiltMode);
    test_attr!(pan_tilt_mode, "PanTiltMode", FixtureChannel3Attribute::PanTiltMode);
    test_attr!(position_modes, "PositionModes", FixtureChannel3Attribute::PositionModes);
    test_attr!(gobo_n_wheel_mode, "Gobo1WheelMode", FixtureChannel3Attribute::GoboWheelMode(1));
    test_attr!(gobo_wheel_shortcut_mode, "GoboWheelShortcutMode", FixtureChannel3Attribute::GoboWheelShortcutMode);
    test_attr!(animation_wheel_n_mode, "AnimationWheel1Mode", FixtureChannel3Attribute::AnimationWheelMode(1));
    test_attr!(animation_wheel_shortcut_mode, "AnimationWheelShortcutMode", FixtureChannel3Attribute::AnimationWheelShortcutMode);
    test_attr!(color_n_mode, "Color1Mode", FixtureChannel3Attribute::ColorMode(1));
    test_attr!(color_wheel_shortcut_mode, "ColorWheelShortcutMode", FixtureChannel3Attribute::ColorWheelShortcutMode);
    test_attr!(cyan_mode, "CyanMode", FixtureChannel3Attribute::CyanMode);
    test_attr!(magenta_mode, "MagentaMode", FixtureChannel3Attribute::MagentaMode);
    test_attr!(yellow_mode, "YellowMode", FixtureChannel3Attribute::YellowMode);
    test_attr!(color_mix_mode, "ColorMixMode", FixtureChannel3Attribute::ColorMixMode);
    test_attr!(chromatic_mode, "ChromaticMode", FixtureChannel3Attribute::ChromaticMode);
    test_attr!(color_calibration_mode, "ColorCalibrationMode", FixtureChannel3Attribute::ColorCalibrationMode);
    test_attr!(color_consistency, "ColorConsistency", FixtureChannel3Attribute::ColorConsistency);
    test_attr!(color_control, "ColorControl", FixtureChannel3Attribute::ColorControl);
    test_attr!(color_model_mode, "ColorModelMode", FixtureChannel3Attribute::ColorModelMode);
    test_attr!(color_settings_reset, "ColorSettingsReset", FixtureChannel3Attribute::ColorSettingsReset);
    test_attr!(color_uniformity, "ColorUniformity", FixtureChannel3Attribute::ColorUniformity);
    test_attr!(cri_mode, "CRIMode", FixtureChannel3Attribute::CriMode);
    test_attr!(custom_color, "CustomColor", FixtureChannel3Attribute::CustomColor);
    test_attr!(uv_stability, "UVStability", FixtureChannel3Attribute::UvStability);
    test_attr!(wavelength_correction, "WavelengthCorrection", FixtureChannel3Attribute::WavelengthCorrection);
    test_attr!(white_count, "WhiteCount", FixtureChannel3Attribute::WhiteCount);
    test_attr!(strobe_mode, "StrobeMode", FixtureChannel3Attribute::StrobeMode);
    test_attr!(zoom_mode, "ZoomMode", FixtureChannel3Attribute::ZoomMode);
    test_attr!(focus_mode, "FocusMode", FixtureChannel3Attribute::FocusMode);
    test_attr!(iris_mode, "IrisMode", FixtureChannel3Attribute::IrisMode);
    test_attr!(fan_n_mode, "Fan1Mode", FixtureChannel3Attribute::FanMode(1));
    test_attr!(follow_spot_mode, "FollowSpotMode", FixtureChannel3Attribute::FollowSpotMode);
    test_attr!(beam_effect_index_rotate_mode, "BeamEffectIndexRotateMode", FixtureChannel3Attribute::BeamEffectIndexRotateMode);
    test_attr!(intensity_m_speed, "IntensityMSpeed", FixtureChannel3Attribute::IntensityMSpeed);
    test_attr!(position_m_speed, "PositionMSpeed", FixtureChannel3Attribute::PositionMSpeed);
    test_attr!(color_mix_m_speed, "ColorMixMSpeed", FixtureChannel3Attribute::ColorMixMSpeed);
    test_attr!(color_wheel_select_m_speed, "ColorWheelSelectMSpeed", FixtureChannel3Attribute::ColorWheelSelectMSpeed);
    test_attr!(gobo_wheel_n_m_speed, "GoboWheel1MSpeed", FixtureChannel3Attribute::GoboWheelMSpeed(1));
    test_attr!(iris_m_speed, "IrisMSpeed", FixtureChannel3Attribute::IrisMSpeed);
    test_attr!(prism_n_m_speed, "Prism1MSpeed", FixtureChannel3Attribute::PrismMSpeed(1));
    test_attr!(focus_m_speed, "FocusMSpeed", FixtureChannel3Attribute::FocusMSpeed);
    test_attr!(frost_n_m_speed, "Frost1MSpeed", FixtureChannel3Attribute::FrostMSpeed(1));
    test_attr!(zoom_m_speed, "ZoomMSpeed", FixtureChannel3Attribute::ZoomMSpeed);
    test_attr!(frame_m_speed, "FrameMSpeed", FixtureChannel3Attribute::FrameMSpeed);
    test_attr!(global_m_speed, "GlobalMSpeed", FixtureChannel3Attribute::GlobalMSpeed);
    test_attr!(reflector_adjust, "ReflectorAdjust", FixtureChannel3Attribute::ReflectorAdjust);
    test_attr!(fixture_global_reset, "FixtureGlobalReset", FixtureChannel3Attribute::FixtureGlobalReset);
    test_attr!(dimmer_reset, "DimmerReset", FixtureChannel3Attribute::DimmerReset);
    test_attr!(shutter_reset, "ShutterReset", FixtureChannel3Attribute::ShutterReset);
    test_attr!(beam_reset, "BeamReset", FixtureChannel3Attribute::BeamReset);
    test_attr!(color_mix_reset, "ColorMixReset", FixtureChannel3Attribute::ColorMixReset);
    test_attr!(color_wheel_reset, "ColorWheelReset", FixtureChannel3Attribute::ColorWheelReset);
    test_attr!(focus_reset, "FocusReset", FixtureChannel3Attribute::FocusReset);
    test_attr!(frame_reset, "FrameReset", FixtureChannel3Attribute::FrameReset);
    test_attr!(gobo_wheel_reset, "GoboWheelReset", FixtureChannel3Attribute::GoboWheelReset);
    test_attr!(intensity_reset, "IntensityReset", FixtureChannel3Attribute::IntensityReset);
    test_attr!(iris_reset, "IrisReset", FixtureChannel3Attribute::IrisReset);
    test_attr!(position_reset, "PositionReset", FixtureChannel3Attribute::PositionReset);
    test_attr!(pan_reset, "PanReset", FixtureChannel3Attribute::PanReset);
    test_attr!(tilt_reset, "TiltReset", FixtureChannel3Attribute::TiltReset);
    test_attr!(zoom_reset, "ZoomReset", FixtureChannel3Attribute::ZoomReset);
    test_attr!(ctb_reset, "CTBReset", FixtureChannel3Attribute::CtbReset);
    test_attr!(cto_reset, "CTOReset", FixtureChannel3Attribute::CtoReset);
    test_attr!(ctc_reset, "CTCReset", FixtureChannel3Attribute::CtcReset);
    test_attr!(animation_system_reset, "AnimationSystemReset", FixtureChannel3Attribute::AnimationSystemReset);
    test_attr!(fixture_calibration_reset, "FixtureCalibrationReset", FixtureChannel3Attribute::FixtureCalibrationReset);
    test_attr!(function, "Function", FixtureChannel3Attribute::Function);
    test_attr!(lamp_control, "LampControl", FixtureChannel3Attribute::LampControl);
    test_attr!(display_intensity, "DisplayIntensity", FixtureChannel3Attribute::DisplayIntensity);
    test_attr!(dmx_input, "DMXInput", FixtureChannel3Attribute::DmxInput);
    test_attr!(no_feature, "NoFeature", FixtureChannel3Attribute::NoFeature);
    test_attr!(blower_n, "Blower1", FixtureChannel3Attribute::Blower(1));
    test_attr!(fan_n, "Fan1", FixtureChannel3Attribute::Fan(1));
    test_attr!(fog_n, "Fog1", FixtureChannel3Attribute::Fog(1));
    test_attr!(haze_n, "Haze1", FixtureChannel3Attribute::Haze(1));
    test_attr!(lamp_power_mode, "LampPowerMode", FixtureChannel3Attribute::LampPowerMode);
    test_attr!(fans, "Fans", FixtureChannel3Attribute::Fans);
    test_attr!(blade_n_a, "Blade1A", FixtureChannel3Attribute::BladeA(1));
    test_attr!(blade_n_b, "Blade1B", FixtureChannel3Attribute::BladeB(1));
    test_attr!(blade_n_rot, "Blade1Rot", FixtureChannel3Attribute::BladeRot(1));
    test_attr!(shaper_rot, "ShaperRot", FixtureChannel3Attribute::ShaperRot);
    test_attr!(shaper_macros, "ShaperMacros", FixtureChannel3Attribute::ShaperMacros);
    test_attr!(shaper_macros_speed, "ShaperMacrosSpeed", FixtureChannel3Attribute::ShaperMacrosSpeed);
    test_attr!(blade_soft_n_a, "BladeSoft1A", FixtureChannel3Attribute::BladeSoftA(1));
    test_attr!(blade_soft_n_b, "BladeSoft1B", FixtureChannel3Attribute::BladeSoftB(1));
    test_attr!(key_stone_n_a, "KeyStone1A", FixtureChannel3Attribute::KeyStoneA(1));
    test_attr!(key_stone_n_b, "KeyStone1B", FixtureChannel3Attribute::KeyStoneB(1));
    test_attr!(video, "Video", FixtureChannel3Attribute::Video);
    test_attr!(video_effect_n_type, "VideoEffect1Type", FixtureChannel3Attribute::VideoEffectType(1));
    test_attr!(video_effect_n_parameter_m, "VideoEffect1Parameter2", FixtureChannel3Attribute::VideoEffectParameter(1, 2));
    test_attr!(video_camera_n, "VideoCamera1", FixtureChannel3Attribute::VideoCamera(1));
    test_attr!(video_sound_volume_n, "VideoSoundVolume1", FixtureChannel3Attribute::VideoSoundVolume(1));
    test_attr!(video_blend_mode, "VideoBlendMode", FixtureChannel3Attribute::VideoBlendMode);
    test_attr!(input_source, "InputSource", FixtureChannel3Attribute::InputSource);
    test_attr!(field_of_view, "FieldOfView", FixtureChannel3Attribute::FieldOfView);

    #[test]
    fn custom() {
        let attribute = FixtureChannel3Attribute::from_str("CustomAttribute").unwrap();
        let found = attribute.to_string();
        assert_eq!(found.as_str(), "CustomAttribute");
    }
}
