#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MidiInOutDeviceMode {
    Input,
    Output,
    Both,
}

impl MidiInOutDeviceMode {
    pub fn is_input(&self) -> bool {
        matches!(self, MidiInOutDeviceMode::Input | MidiInOutDeviceMode::Both)
    }

    pub fn is_output(&self) -> bool {
        matches!(
            self,
            MidiInOutDeviceMode::Output | MidiInOutDeviceMode::Both
        )
    }
}
