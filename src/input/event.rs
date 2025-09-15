use crate::{
    input::control::{button::DemexInputButton, fader::DemexInputFader},
    parser::nodes::fixture_selector::FixtureSelector,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemexInputDeviceEvent {
    ExecutorGo(u32),
    ExecutorStop(u32),
    ExecutorFaderValueChanged(u32),

    GrandmasterFaderValueChanged,

    SpeedmasterFaderValueChanged(u32),

    FixtureSelector(FixtureSelector),
}

#[derive(Debug, Clone)]
pub enum DemexInputDeviceFaderUpdate {
    FaderValueChange(f32),
}

impl Default for DemexInputDeviceFaderUpdate {
    fn default() -> Self {
        DemexInputDeviceFaderUpdate::FaderValueChange(0.0)
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

    Button {
        id: u32,
        button: &'a DemexInputButton,
        update: DemexInputDeviceButtonUpdate,
    },
}
