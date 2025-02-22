use crate::{
    fixture::{
        handler::error::FixtureHandlerError, presets::error::PresetHandlerError,
        timing::error::TimingHandlerError, updatables::error::UpdatableHandlerError,
    },
    parser::nodes::fixture_selector::FixtureSelectorError,
};

#[derive(Debug)]
pub enum DemexInputDeviceError {
    ButtonNotFound(u32),
    FaderNotFound(u32),

    ButtonNotInProfile,
    FaderNotInProfile,

    InputDeviceNotFound(String),
    InputDeviceIdxNotFound(usize),
    OperationNotSupported,

    FixtureHandlerError(FixtureHandlerError),
    PresetHandlerError(PresetHandlerError),
    UpdatableHandlerError(UpdatableHandlerError),
    FixtureSelectorError(FixtureSelectorError),
    TimingHandlerError(TimingHandlerError),

    ButtonAlreadyAssigned(u32),
    FaderAlreadyAssigned(u32),

    ButtonNotAssigned(u32),
    FaderNotAssigned(u32),

    NotImplemented,

    MpscSendError,
    MidirError(Box<dyn std::error::Error>),
}

impl std::error::Error for DemexInputDeviceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for DemexInputDeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ButtonNotFound(id) => write!(f, "Button with id {} not found", id),
            Self::FaderNotFound(id) => write!(f, "Fader with id {} not found", id),

            Self::ButtonNotInProfile => write!(f, "Button not in profile"),
            Self::FaderNotInProfile => write!(f, "Fader not in profile"),

            Self::InputDeviceNotFound(name) => write!(f, "Input device \"{}\" not found", name),
            Self::InputDeviceIdxNotFound(idx) => {
                write!(f, "Input device with index {} not found", idx)
            }
            Self::OperationNotSupported => write!(f, "Operation not supported"),

            Self::FixtureHandlerError(err) => write!(f, "Fixture handler error: {}", err),
            Self::PresetHandlerError(err) => write!(f, "Preset handler error: {}", err),
            Self::UpdatableHandlerError(err) => write!(f, "Updatable handler error: {}", err),
            Self::FixtureSelectorError(err) => write!(f, "Fixture selector error: {}", err),
            Self::TimingHandlerError(err) => write!(f, "Timing handler error: {}", err),

            Self::ButtonAlreadyAssigned(id) => write!(f, "Button with id {} already assigned", id),
            Self::FaderAlreadyAssigned(id) => write!(f, "Fader with id {} already assigned", id),

            Self::ButtonNotAssigned(id) => write!(f, "Button with id {} not assigned", id),
            Self::FaderNotAssigned(id) => write!(f, "Fader with id {} not assigned", id),

            Self::NotImplemented => write!(f, "Not implemented"),

            Self::MpscSendError => write!(f, "Mpsc send error"),
            Self::MidirError(err) => write!(f, "Midir error: {}", err),
        }
    }
}
