use crate::{
    event::DemexEvent,
    input::{DemexInputDeviceUpdateArgs, error::DemexInputDeviceError},
};

pub mod button;
pub mod encoder;
pub mod fader;
pub mod motorized;

pub trait DemexInputDeviceControlDelegate {
    type Update;

    fn map_event(
        &self,
        args: DemexInputDeviceUpdateArgs,
        event: &DemexEvent,
    ) -> Result<Option<Self::Update>, DemexInputDeviceError>;
}

pub trait DemexInputDeviceControlAssignmentDelegate {
    type Control: DemexInputDeviceControlDelegate;

    fn assign(
        self,
    ) -> Result<
        (
            Self::Control,
            Option<<Self::Control as DemexInputDeviceControlDelegate>::Update>,
        ),
        DemexInputDeviceError,
    >;
}
