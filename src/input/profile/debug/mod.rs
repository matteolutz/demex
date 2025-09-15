use crate::{
    fixture::patch::Patch,
    input::{message::DemexInputDeviceMessage, DemexInputDeviceProfile},
    ui::context::EncoderChannels,
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
    fn update_out(
        &mut self,
        _device_config: &crate::input::device::DemexInputDeviceConfig,
        _fixture_handler: &crate::fixture::handler::FixtureHandler,
        _preset_handler: &crate::fixture::presets::PresetHandler,
        _updatable_handler: &crate::fixture::updatables::UpdatableHandler,
        _timing_handler: &crate::fixture::timing::TimingHandler,
        _global_fixture_selection: &Option<crate::fixture::selection::FixtureSelection>,
        _: &Patch,
        _: Option<&EncoderChannels>,
    ) -> Result<(), crate::input::error::DemexInputDeviceError> {
        Ok(())
    }

    fn poll(
        &self,
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
