use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::ActionRunArgs,
    input::{
        control::{
            DemexInputDeviceControlAssignmentDelegate,
            button::{DemexInputButton, DemexInputButtonAssignment},
            encoder::DemexInputEncoder,
            fader::{DemexInputFader, DemexInputFaderAssignment},
        },
        error::DemexInputDeviceError,
        event::DemexInputDeviceControlUpdate,
        profile::{behringer::BehringerXTouchCompactDeviceProfile, debug::DebugDeviceProfile},
    },
};

use super::{
    DemexInputDeviceProfile,
    profile::{
        DemexInputDeviceProfileType, akai::ApcMiniMk2InputDeviceProfile,
        midi_timecode::MidiTimecodeProfile,
    },
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct DemexInputDeviceControls {
    buttons: HashMap<u32, DemexInputButton>,
    faders: HashMap<u32, DemexInputFader>,

    #[serde(default)]
    encoders: HashMap<u32, DemexInputEncoder>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DemexInputDeviceConfig {
    #[serde(default)]
    controls: DemexInputDeviceControls,

    profile_type: DemexInputDeviceProfileType,
}

impl Deref for DemexInputDeviceConfig {
    type Target = DemexInputDeviceControls;

    fn deref(&self) -> &Self::Target {
        &self.controls
    }
}
impl DerefMut for DemexInputDeviceConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.controls
    }
}

impl DemexInputDeviceConfig {
    pub fn buttons(&self) -> &HashMap<u32, DemexInputButton> {
        &self.controls.buttons
    }

    pub fn buttons_mut(&mut self) -> &mut HashMap<u32, DemexInputButton> {
        &mut self.controls.buttons
    }

    pub fn faders(&self) -> &HashMap<u32, DemexInputFader> {
        &self.controls.faders
    }

    pub fn faders_mut(&mut self) -> &mut HashMap<u32, DemexInputFader> {
        &mut self.controls.faders
    }

    pub fn encoders(&self) -> &HashMap<u32, DemexInputEncoder> {
        &self.controls.encoders
    }

    pub fn encoders_mut(&mut self) -> &mut HashMap<u32, DemexInputEncoder> {
        &mut self.controls.encoders
    }

    pub fn profile_type(&self) -> &DemexInputDeviceProfileType {
        &self.profile_type
    }

    pub fn into_device(
        self,
        args: &ActionRunArgs,
    ) -> Result<DemexInputDevice, DemexInputDeviceError> {
        let profile: Box<dyn DemexInputDeviceProfile> = match self.profile_type {
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

        let mut device = DemexInputDevice::new(self.profile_type, profile);
        device.apply_controls(self.controls, args)?;

        Ok(device)
    }
}

#[derive(Debug)]
pub struct DemexInputDevice {
    pub(crate) profile: Box<dyn DemexInputDeviceProfile>,
    pub(crate) config: DemexInputDeviceConfig,
}

impl DemexInputDevice {
    pub fn new(
        profile_type: DemexInputDeviceProfileType,
        profile: Box<dyn DemexInputDeviceProfile>,
    ) -> Self {
        Self {
            profile,
            config: DemexInputDeviceConfig {
                profile_type,
                controls: DemexInputDeviceControls::default(),
            },
        }
    }

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

    pub fn apply_controls(
        &mut self,
        controls: DemexInputDeviceControls,
        args: &ActionRunArgs,
    ) -> Result<(), DemexInputDeviceError> {
        self.config.controls = DemexInputDeviceControls::default();

        for (button_id, button) in controls.buttons {
            let assignment = DemexInputButtonAssignment::from_control(button, &args)?;
            self.assign_button(button_id, assignment)?;
        }

        for (fader_id, fader) in controls.faders {
            let assignment = DemexInputFaderAssignment::from_control(fader, &args)?;
            self.assign_fader(fader_id, assignment)?;
        }

        // TODO: encoders

        Ok(())
    }
}
