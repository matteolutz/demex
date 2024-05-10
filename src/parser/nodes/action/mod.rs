use open_dmx::DMXSerial;

use self::{error::ActionRunError, result::ActionRunResult};

use super::fixture_selector::FixtureSelector;

pub mod error;
pub mod result;

#[derive(Debug)]
pub enum Action {
    SetIntensity(FixtureSelector, u8),
}

impl Action {
    pub fn run(&self, dmx: &mut DMXSerial) -> Result<ActionRunResult, ActionRunError> {
        match self {
            Self::SetIntensity(fixture_selector, intensity) => {
                self.run_set_intensity(dmx, fixture_selector, *intensity)
            }
        }
    }

    pub fn run_set_intensity(
        &self,
        dmx: &mut DMXSerial,
        fixture_selector: &FixtureSelector,
        intensity: u8,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixture_dmx_channels = fixture_selector.get_dmx_channels();
        for fixture_channel in fixture_dmx_channels {
            dmx.set_channel(fixture_channel, intensity)
                .map_err(|err| ActionRunError::DMXChannelValidityError(err))?;
        }

        Ok(ActionRunResult {
            should_update: true,
        })
    }
}
