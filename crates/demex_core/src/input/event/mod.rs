use crate::input::control::{
    button::DemexInputButton, encoder::DemexInputEncoder, fader::DemexInputFader,
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
