use crate::fixture::updatables::error::UpdatableHandlerError;

#[derive(Debug)]
pub enum DemexInputDeviceError {
    ButtonNotFound(u32),
    FaderNotFound(u32),

    InputDeviceNotFound(String),
    UpdatableHandlerError(UpdatableHandlerError),
    OperationNotSupported,
    MpscSendError,
    MidirInitError(midir::InitError),
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

            Self::InputDeviceNotFound(name) => write!(f, "Input device \"{}\" not found", name),
            Self::UpdatableHandlerError(err) => write!(f, "Updatable handler error: {}", err),
            Self::OperationNotSupported => write!(f, "Operation not supported"),
            Self::MpscSendError => write!(f, "Mpsc send error"),
            Self::MidirInitError(err) => write!(f, "Midir init error: {}", err),
        }
    }
}
