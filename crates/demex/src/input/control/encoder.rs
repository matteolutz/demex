use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        EncoderChannels, handler::FixtureHandler, patch::Patch, presets::PresetHandler,
        timing::TimingHandler, updatables::UpdatableHandler,
    },
    input::{
        DemexInputDeviceUpdateArgs,
        control::DemexInputDeviceControlTrait,
        encoder::{get_global_encoder_value, handle_global_encoder_change},
        error::DemexInputDeviceError,
        event::{DemexInputDeviceEncoderUpdate, DemexInputDeviceEvent},
    },
    parser::nodes::fixture_selector::FixtureSelectorContext,
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
        fixture_handler: &mut FixtureHandler,
        encoder_channels: Option<&EncoderChannels>,
        _preset_handler: &PresetHandler,
        _updatable_handler: &mut UpdatableHandler,
        _timing_handler: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<Option<DemexInputDeviceEvent>, DemexInputDeviceError> {
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

                Some(DemexInputDeviceEvent::GlobalEncoderValueChanged(
                    *encoder_idx,
                ))
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
        event: &crate::input::event::DemexInputDeviceEvent,
    ) -> Result<Option<DemexInputDeviceEncoderUpdate>, crate::input::error::DemexInputDeviceError>
    {
        let update = match self {
            Self::GlobalEncoder { encoder_idx } => {
                if matches!(event, DemexInputDeviceEvent::GlobalEncoderValueChanged(event_encoder_idx) if event_encoder_idx == encoder_idx)
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
