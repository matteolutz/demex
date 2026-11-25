use crate::{
    event::DemexEvent,
    input::{
        DemexInputDeviceUpdateArgs,
        control::{
            DemexInputDeviceControlDelegate, button::DemexInputButton, encoder::DemexInputEncoder,
            fader::DemexInputFader,
        },
        device::DemexInputDeviceConfig,
    },
};

pub mod handler;

#[derive(Debug, Clone)]
pub enum DemexInputDeviceFaderUpdate {
    FaderValueChange(f32),
}

impl Default for DemexInputDeviceFaderUpdate {
    fn default() -> Self {
        Self::FaderValueChange(0.0)
    }
}

#[derive(Debug, Clone)]
pub enum DemexInputDeviceEncoderUpdate {
    EncoderValueChange(f32),
}

impl Default for DemexInputDeviceEncoderUpdate {
    fn default() -> Self {
        Self::EncoderValueChange(0.0)
    }
}

#[derive(Debug, Clone, Default)]
pub enum DemexInputDeviceButtonUpdate {
    ButtonActive,

    #[default]
    ButtonInactive,
}

#[derive(Debug, Clone)]
pub enum DemexInputDeviceControlUpdate<'a> {
    Fader {
        id: u32,
        fader: &'a DemexInputFader,
        update: DemexInputDeviceFaderUpdate,
    },

    Encoder {
        id: u32,
        encoder: &'a DemexInputEncoder,
        update: DemexInputDeviceEncoderUpdate,
    },

    // This encoder is not user assignable
    GlobalEncoder {
        id: u32,
        update: DemexInputDeviceEncoderUpdate,
    },

    Button {
        id: u32,
        button: &'a DemexInputButton,
        update: DemexInputDeviceButtonUpdate,
    },
}

pub trait DemexInputDeviceConfigExt {
    fn map_events<'a>(
        &'a self,
        args: DemexInputDeviceUpdateArgs<'a>,
        events: impl IntoIterator<Item = &'a DemexEvent>,
    ) -> Vec<DemexInputDeviceControlUpdate<'a>>;
}

impl DemexInputDeviceConfigExt for DemexInputDeviceConfig {
    fn map_events<'a>(
        &'a self,
        args: DemexInputDeviceUpdateArgs<'a>,
        events: impl IntoIterator<Item = &'a DemexEvent>,
    ) -> Vec<DemexInputDeviceControlUpdate<'a>> {
        events
            .into_iter()
            .flat_map(|event| {
                let mut device_events = vec![];

                // buttons
                for (id, button) in self.buttons() {
                    if let Some(update) = button.map_event(args.clone(), event).ok().flatten() {
                        device_events.push(DemexInputDeviceControlUpdate::Button {
                            button,
                            update,
                            id: *id,
                        });
                    }
                }

                // faders
                for (id, fader) in self.faders() {
                    if let Some(update) = fader.map_event(args.clone(), event).ok().flatten() {
                        device_events.push(DemexInputDeviceControlUpdate::Fader {
                            fader,
                            update,
                            id: *id,
                        });
                    }
                }

                // TODO: encoders
                // TODO: also clean this up

                device_events
            })
            .collect()
    }
}
