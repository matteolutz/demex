use crate::fixture::updatables::error::UpdatableHandlerError;

#[derive(Debug)]
pub enum DemexInputDeviceError {
    ButtonNotFound(u32),
    RusbError(rusb::Error),
    InputDeviceNotFound(String),
    UpdatableHandlerError(UpdatableHandlerError),
    OperationNotSupported,
    MpscSendError,
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
            Self::RusbError(err) => write!(f, "Rusb error: {}", err),
            Self::InputDeviceNotFound(name) => write!(f, "Input device \"{}\" not found", name),
            Self::UpdatableHandlerError(err) => write!(f, "Updatable handler error: {}", err),
            Self::OperationNotSupported => write!(f, "Operation not supported"),
            Self::MpscSendError => write!(f, "Mpsc send error"),
        }
    }
}
