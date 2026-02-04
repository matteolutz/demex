use crate::input::{
    DemexInputDeviceProfile, DemexInputDeviceUpdateArgs, message::DemexInputDeviceMessage,
};

const ENABLED: bool = false;
const MESSAGES_TO_SEND: &[DemexInputDeviceMessage] =
    &[DemexInputDeviceMessage::GlobalEncoderValueChanged {
        encoder_idx: 0,
        value: 0.69,
    }];

#[derive(Debug)]
pub struct DebugDeviceProfile {}

impl DebugDeviceProfile {
    pub fn new() -> Self {
        Self {}
    }
}

impl DemexInputDeviceProfile for DebugDeviceProfile {
    fn handle_events(
        &mut self,
        _events: &[crate::input::event::DemexInputDeviceControlUpdate],
    ) -> Result<(), crate::input::error::DemexInputDeviceError> {
        Ok(())
    }

    fn tick(
        &mut self,
        _: DemexInputDeviceUpdateArgs,
    ) -> Result<(), crate::input::error::DemexInputDeviceError> {
        Ok(())
    }

    fn poll(
        &mut self,
    ) -> Result<
        Vec<crate::input::message::DemexInputDeviceMessage>,
        crate::input::error::DemexInputDeviceError,
    > {
        Ok(if ENABLED {
            MESSAGES_TO_SEND.to_vec()
        } else {
            vec![]
        })
    }

    fn is_enabled(&self) -> bool {
        true
    }
}
