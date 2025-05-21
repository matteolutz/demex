use super::{presets::error::PresetHandlerError, updatables::error::UpdatableHandlerError};

#[derive(Debug)]
pub enum FixtureError {
    FaderProvidesNoValues(u32),
    NoChannelValueSourceFound,
    EmptyPatch,
    DuplicateChannelType,
    InvalidDataLength,
    NoFunctionAccess,
    FixtureTypeNotFound(String),
    FixtureTypeModeNotFound(String, u32),

    GdtfFixtureTypeNotFound(uuid::Uuid),
    GdtfFixtureDmxModeNotFound(String),
    GdtfChannelValueNotConvertable(String),
    GdtfMaxDmxOffsetNotFound,
    GdtfChannelNotFound(String),
    GdtfChannelValueNotFound(String),
    GdtfNoChannelForAttributeFound(String),
    GdtfChannelHasNoAttribute(String),
    GdtfChannelFunctionMismatch(usize, usize),
    GdtfAtributeHasNoName,

    NoDisplayColor(u32),
    PresetHandlerError(Box<PresetHandlerError>),
    UpdatableHandlerError(Box<UpdatableHandlerError>),
}

impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FaderProvidesNoValues(fader_id) => {
                write!(f, "Fader {} provides no values", fader_id)
            }
            Self::NoChannelValueSourceFound => write!(f, "No channel value source found"),
            Self::EmptyPatch => write!(f, "Patch is empty"),
            Self::DuplicateChannelType => write!(f, "Duplicate channel type"),
            Self::InvalidDataLength => write!(f, "Invalid data length"),
            Self::NoFunctionAccess => write!(f, "Tried to access values for a NoFunction channel"),
            Self::FixtureTypeNotFound(s) => write!(f, "Fixture type {} not found", s),
            Self::FixtureTypeModeNotFound(fixture_type, fixture_mode) => {
                write!(
                    f,
                    "Fixture type mode {} for type {} not found",
                    fixture_type, fixture_mode
                )
            }
            Self::NoDisplayColor(fixture_id) => {
                write!(f, "Fixture {} has no color feature", fixture_id)
            }
            Self::PresetHandlerError(err) => write!(f, "Preset handler error: {}", err),
            Self::UpdatableHandlerError(err) => write!(f, "Updatable handler error: {}", err),

            Self::GdtfFixtureDmxModeNotFound(mode) => {
                write!(f, "GDTF fixture DMX mode {} not found", mode)
            }
            Self::GdtfChannelValueNotConvertable(dmx_channel_name) => {
                write!(
                    f,
                    "GDTF channel value for channel {} not convertable",
                    dmx_channel_name
                )
            }
            Self::GdtfMaxDmxOffsetNotFound => {
                write!(f, "GDTF fixture DMX mode has no max offset")
            }
            Self::GdtfChannelNotFound(channel) => {
                write!(f, "GDTF channel {} not found", channel)
            }
            Self::GdtfFixtureTypeNotFound(type_id) => {
                write!(f, "GDTF fixture type with id {} not found", type_id)
            }
            Self::GdtfChannelValueNotFound(channel) => {
                write!(f, "GDTF value for channel {} not found", channel)
            }
            Self::GdtfNoChannelForAttributeFound(attribute) => {
                write!(f, "GDTF no channel for attribute {} found", attribute)
            }
            Self::GdtfChannelHasNoAttribute(channel) => {
                write!(f, "GDTF channel {} has no attribute", channel)
            }
            Self::GdtfChannelFunctionMismatch(channel1, channel2) => {
                write!(
                    f,
                    "GDTF channel function idx mismatch: {} != {}",
                    channel1, channel2
                )
            }
            Self::GdtfAtributeHasNoName => {
                write!(f, "GDTF attribute has no name")
            }
        }
    }
}

impl std::error::Error for FixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
