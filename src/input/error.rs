use crate::fixture::updatables::error::UpdatableHandlerError;

#[derive(Debug)]
pub enum DemexInputDeviceError {
    ButtonNotFound(u32),
    UpdatableHandlerError(UpdatableHandlerError),
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
            Self::UpdatableHandlerError(err) => write!(f, "Updatable handler error: {}", err),
        }
    }
}
