use crate::{
    event::DemexEvent,
    input::{
        DemexInputDeviceUpdateArgs,
        control::{button::DemexInputButtonAssignment, fader::DemexInputFaderAssignment},
        error::DemexInputDeviceError,
    },
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

pub struct DemexInputControlAssignmentResult<T: DemexInputDeviceControlDelegate> {
    pub control: T,
    pub init_event: Option<T::Update>,
}

pub trait DemexInputDeviceControlAssignmentDelegate {
    type Control: DemexInputDeviceControlDelegate;

    fn assign(
        self,
    ) -> Result<DemexInputControlAssignmentResult<Self::Control>, DemexInputDeviceError>;
}

#[derive(Debug, Clone)]
pub enum DemexInputDeviceControlAssignment {
    Button {
        device_idx: usize,
        button_id: u32,
        assignment: DemexInputButtonAssignment,
    },

    Fader {
        device_idx: usize,
        fader_id: u32,
        assignment: DemexInputFaderAssignment,
    },
}

#[derive(Debug, Clone)]
pub enum DemexInputDeviceControlUnassignment {
    Button { device_idx: usize, button_id: u32 },

    Fader { device_idx: usize, fader_id: u32 },
}
