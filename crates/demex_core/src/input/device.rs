use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::input::{
    control::{
        DemexInputDeviceControlAssignmentDelegate,
        button::{DemexInputButton, DemexInputButtonAssignment},
        encoder::DemexInputEncoder,
        fader::{DemexInputFader, DemexInputFaderAssignment},
    },
    error::DemexInputDeviceError,
    event::DemexInputDeviceControlUpdate,
    profile::{behringer::BehringerXTouchCompactDeviceProfile, debug::DebugDeviceProfile},
};

use super::{
    DemexInputDeviceProfile,
    profile::{
        DemexInputDeviceProfileType, akai::ApcMiniMk2InputDeviceProfile,
        midi_timecode::MidiTimecodeProfile,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DemexInputDeviceConfig {
    buttons: HashMap<u32, DemexInputButton>,
    faders: HashMap<u32, DemexInputFader>,

    #[serde(default)]
    encoders: HashMap<u32, DemexInputEncoder>,

    profile_type: DemexInputDeviceProfileType,
}

impl DemexInputDeviceConfig {
    pub fn new(
        buttons: HashMap<u32, DemexInputButton>,
        faders: HashMap<u32, DemexInputFader>,
        encoders: HashMap<u32, DemexInputEncoder>,
        profile_type: DemexInputDeviceProfileType,
    ) -> Self {
        Self {
            buttons,
            faders,
            encoders,
            profile_type,
        }
    }

    pub fn buttons(&self) -> &HashMap<u32, DemexInputButton> {
        &self.buttons
    }

    pub fn buttons_mut(&mut self) -> &mut HashMap<u32, DemexInputButton> {
        &mut self.buttons
    }

    pub fn faders(&self) -> &HashMap<u32, DemexInputFader> {
        &self.faders
    }

    pub fn faders_mut(&mut self) -> &mut HashMap<u32, DemexInputFader> {
        &mut self.faders
    }

    pub fn encoders(&self) -> &HashMap<u32, DemexInputEncoder> {
        &self.encoders
    }

    pub fn encoders_mut(&mut self) -> &mut HashMap<u32, DemexInputEncoder> {
        &mut self.encoders
    }

    pub fn profile_type(&self) -> &DemexInputDeviceProfileType {
        &self.profile_type
    }
}

#[derive(Debug)]
pub struct DemexInputDevice {
    pub(crate) profile: Box<dyn DemexInputDeviceProfile>,
    pub(crate) config: DemexInputDeviceConfig,
}

impl DemexInputDevice {
    pub fn profile(&self) -> &dyn DemexInputDeviceProfile {
        self.profile.as_ref()
    }

    pub fn profile_mut(&mut self) -> &mut dyn DemexInputDeviceProfile {
        self.profile.as_mut()
    }

    pub fn config(&self) -> &DemexInputDeviceConfig {
        &self.config
    }

    pub fn assign_button(
        &mut self,
        button_id: u32,
        assignment: DemexInputButtonAssignment,
    ) -> Result<(), DemexInputDeviceError> {
        if self.config.buttons.contains_key(&button_id) {
            Err(DemexInputDeviceError::ButtonAlreadyAssigned(button_id))
        } else {
            let assignment_result = assignment.assign()?;
            let button = self
                .config
                .buttons
                .entry(button_id)
                .or_insert(assignment_result.control);

            if let Some(init_event) = assignment_result.init_event {
                self.profile
                    .handle_events(&[DemexInputDeviceControlUpdate::Button {
                        id: button_id,
                        button,
                        update: init_event,
                    }])?;
            }

            Ok(())
        }
    }

    pub fn unassign_button(&mut self, button_id: u32) -> Result<(), DemexInputDeviceError> {
        if !self.config.buttons.contains_key(&button_id) {
            Err(DemexInputDeviceError::ButtonNotAssigned(button_id))
        } else {
            self.config.buttons.remove(&button_id);
            Ok(())
        }
    }

    pub fn assign_fader(
        &mut self,
        fader_id: u32,
        assignment: DemexInputFaderAssignment,
    ) -> Result<(), DemexInputDeviceError> {
        if self.config.faders.contains_key(&fader_id) {
            Err(DemexInputDeviceError::FaderAlreadyAssigned(fader_id))
        } else {
            let assignment_result = assignment.assign()?;
            let fader = self
                .config
                .faders
                .entry(fader_id)
                .or_insert(assignment_result.control);

            if let Some(init_event) = assignment_result.init_event {
                self.profile
                    .handle_events(&[DemexInputDeviceControlUpdate::Fader {
                        id: fader_id,
                        fader,
                        update: init_event,
                    }])?;
            }

            Ok(())
        }
    }

    pub fn unassign_fader(&mut self, fader_id: u32) -> Result<(), DemexInputDeviceError> {
        if !self.config.faders.contains_key(&fader_id) {
            Err(DemexInputDeviceError::FaderNotAssigned(fader_id))
        } else {
            self.config.faders.remove(&fader_id);
            Ok(())
        }
    }
}

impl From<DemexInputDeviceConfig> for DemexInputDevice {
    fn from(value: DemexInputDeviceConfig) -> Self {
        let profile: Box<dyn DemexInputDeviceProfile> = match value.profile_type {
            DemexInputDeviceProfileType::ApcMiniMk2 { ref apc_midi } => {
                Box::new(ApcMiniMk2InputDeviceProfile::new(apc_midi.clone()))
            }
            DemexInputDeviceProfileType::MidiTimecode { ref midi_in_device } => {
                Box::new(MidiTimecodeProfile::new(midi_in_device.clone()))
            }
            DemexInputDeviceProfileType::BehringerXTouchCompact { ref xtouch_midi } => Box::new(
                BehringerXTouchCompactDeviceProfile::new(xtouch_midi.clone()),
            ),
            DemexInputDeviceProfileType::Debug => Box::new(DebugDeviceProfile::new()),
        };

        DemexInputDevice {
            config: value,
            profile,
        }
    }
}
