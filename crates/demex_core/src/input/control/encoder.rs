use serde::{Deserialize, Serialize};

use crate::{
    EncoderChannels,
    command::parser::nodes::fixture_selector::FixtureSelectorContext,
    event::DemexEvent,
    input::{
        DemexInputDeviceUpdateArgs,
        control::DemexInputDeviceControlTrait,
        encoder::{get_global_encoder_value, handle_global_encoder_change},
        error::DemexInputDeviceError,
        event::DemexInputDeviceEncoderUpdate,
    },
    patch::Patch,
    presets::PresetHandler,
    state::fixture_state_handler::FixtureStateHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DemexInputEncoder {
    GlobalEncoder { encoder_idx: u32 },
}

impl Default for DemexInputEncoder {
    fn default() -> Self {
        Self::GlobalEncoder { encoder_idx: 0 }
    }
}

impl DemexInputEncoder {
    pub fn handle_change(
        &self,
        value: f32,
        fixture_selector_context: FixtureSelectorContext,
        fixture_handler: &mut FixtureStateHandler,
        encoder_channels: Option<&EncoderChannels>,
        _preset_handler: &PresetHandler,
        _updatable_handler: &mut UpdatableHandler,
        _timing_handler: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<Option<DemexEvent>, DemexInputDeviceError> {
        let event = match self {
            Self::GlobalEncoder { encoder_idx } => {
                handle_global_encoder_change(
                    *encoder_idx,
                    value,
                    fixture_selector_context,
                    fixture_handler,
                    encoder_channels,
                    patch,
                );

                Some(DemexEvent::GlobalEncoderValueChanged(*encoder_idx))
            }
        };

        Ok(event)
    }

    pub fn value(&self, args: DemexInputDeviceUpdateArgs) -> Result<f32, DemexInputDeviceError> {
        match self {
            Self::GlobalEncoder { encoder_idx } => Ok(get_global_encoder_value(
                *encoder_idx,
                FixtureSelectorContext::new(args.global_fixture_selection),
                args.fixture_handler,
                args.preset_handler,
                args.timing_handler,
                args.encoder_channels,
                args.patch,
            )
            .unwrap_or(0.0)),
        }
    }
}

impl DemexInputDeviceControlTrait<DemexInputDeviceEncoderUpdate> for DemexInputEncoder {
    fn should_update(
        &self,
        args: crate::input::DemexInputDeviceUpdateArgs,
        event: &DemexEvent,
    ) -> Result<Option<DemexInputDeviceEncoderUpdate>, crate::input::error::DemexInputDeviceError>
    {
        let update = match self {
            Self::GlobalEncoder { encoder_idx } => {
                if matches!(event, DemexEvent::GlobalEncoderValueChanged(event_encoder_idx) if event_encoder_idx == encoder_idx)
                {
                    let value = get_global_encoder_value(
                        *encoder_idx,
                        FixtureSelectorContext::new(args.global_fixture_selection),
                        args.fixture_handler,
                        args.preset_handler,
                        args.timing_handler,
                        args.encoder_channels,
                        args.patch,
                    );

                    value.map(DemexInputDeviceEncoderUpdate::EncoderValueChange)
                } else {
                    None
                }
            }
        };

        Ok(update)
    }

    fn initial_state(
        &self,
        args: crate::input::DemexInputDeviceUpdateArgs,
    ) -> Result<DemexInputDeviceEncoderUpdate, crate::input::error::DemexInputDeviceError> {
        self.value(args)
            .map(DemexInputDeviceEncoderUpdate::EncoderValueChange)
    }
}
