use crate::{
    event::DemexEvent,
    input::{DemexInputDeviceUpdateArgs, error::DemexInputDeviceError},
};

pub mod button;
pub mod encoder;
pub mod fader;
pub mod motorized;

pub trait DemexInputDeviceControlTrait<T> {
    fn should_update(
        &self,
        args: DemexInputDeviceUpdateArgs,
        event: &DemexEvent,
    ) -> Result<Option<T>, DemexInputDeviceError>;

    fn initial_state(&self, args: DemexInputDeviceUpdateArgs) -> Result<T, DemexInputDeviceError>;
}
