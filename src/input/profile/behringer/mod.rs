use std::sync::mpsc;

use crate::input::{error::DemexInputDeviceError, midi::MidiMessage, DemexInputDeviceProfile};

pub struct BehringerXTouchCompactDeviceProfile {
    #[allow(dead_code)]
    xtouch_midi_name: String,

    rx: mpsc::Receiver<MidiMessage>,
    midi_out: Option<midir::MidiOutputConnection>,
    _midi_in: Option<midir::MidiInputConnection<()>>,
}

impl std::fmt::Debug for BehringerXTouchCompactDeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BehringerXTouchCompactDeviceProfile")
            .field("xtouch_midi_name", &self.xtouch_midi_name)
            .finish()
    }
}

impl BehringerXTouchCompactDeviceProfile {
    fn get_conn_out(
        xtouch_midi_name: &str,
    ) -> Result<midir::MidiOutputConnection, DemexInputDeviceError> {
        Err(DemexInputDeviceError::InputDeviceNotFound(
            xtouch_midi_name.to_owned(),
        ))
    }

    fn get_conn_in(
        xtouch_midi_name: &str,
        tx: mpsc::Sender<MidiMessage>,
    ) -> Result<midir::MidiInputConnection<()>, DemexInputDeviceError> {
        Err(DemexInputDeviceError::InputDeviceNotFound(
            xtouch_midi_name.to_owned(),
        ))
    }
}

impl BehringerXTouchCompactDeviceProfile {
    pub fn new(xtouch_midi_name: String) -> Self {
        let (tx, rx) = mpsc::channel();

        let conn_out = Self::get_conn_out(&xtouch_midi_name);
        let conn_in = Self::get_conn_in(&xtouch_midi_name, tx);

        let mut s = Self {
            xtouch_midi_name,
            rx,
            midi_out: conn_out
                .inspect_err(|err| {
                    log::warn!(
                        "Failed to establish Behringer X-Touch Compact MIDI out connection: {}",
                        err
                    )
                })
                .ok(),
            _midi_in: conn_in
                .inspect_err(|err| {
                    log::warn!(
                        "Failed to establish Behringer X-Touch Compact MIDI in connection: {}",
                        err
                    )
                })
                .ok(),
        };

        if let Err(err) = s.init() {
            log::error!(
                "Failed to initialize Behringer X-Touch Compact device: {}",
                err
            );
        }

        s
    }

    fn init(&mut self) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }
}

impl DemexInputDeviceProfile for BehringerXTouchCompactDeviceProfile {
    fn update_out(
        &mut self,
        device_config: &crate::input::device::DemexInputDeviceConfig,
        preset_handler: &crate::fixture::presets::PresetHandler,
        updatable_handler: &crate::fixture::updatables::UpdatableHandler,
        timing_handler: &crate::fixture::timing::TimingHandler,
        global_fixture_selection: &Option<crate::fixture::selection::FixtureSelection>,
    ) -> Result<(), DemexInputDeviceError> {
        todo!()
    }

    fn poll(
        &self,
    ) -> Result<Vec<crate::input::message::DemexInputDeviceMessage>, DemexInputDeviceError> {
        todo!()
    }

    fn is_enabled(&self) -> bool {
        self.midi_out.is_some()
    }
}
