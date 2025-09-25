use crate::input::{
    error::DemexInputDeviceError, event::DemexInputDeviceEvent, DemexInputDeviceUpdateArgs,
};

pub mod button;
pub mod encoder;
pub mod fader;
pub mod motorized;

pub trait DemexInputDeviceControlTrait<T> {
    fn should_update(
        &self,
        args: DemexInputDeviceUpdateArgs,
        event: &DemexInputDeviceEvent,
    ) -> Result<Option<T>, DemexInputDeviceError>;

    fn initial_state(&self, args: DemexInputDeviceUpdateArgs) -> Result<T, DemexInputDeviceError>;
}
