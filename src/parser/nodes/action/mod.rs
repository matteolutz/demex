use crate::fixture::handler::FixtureHandler;

use self::{error::ActionRunError, result::ActionRunResult};

use super::fixture_selector::FixtureSelector;

pub mod error;
pub mod result;

#[derive(Debug)]
pub enum Action {
    SetIntensity(FixtureSelector, u8),
    GoHome(FixtureSelector),
    GoHomeAll,
    ManSet(FixtureSelector, String, u8),
    FixtureSelector(FixtureSelector),
}

impl Action {
    pub fn run(
        &self,
        fixture_handler: &mut FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            Self::SetIntensity(fixture_selector, intensity) => {
                self.run_set_intensity(fixture_handler, fixture_selector, *intensity)
            }
            Self::GoHome(fixture_selector) => self.run_go_home(fixture_handler, fixture_selector),
            Self::GoHomeAll => self.run_go_home_all(fixture_handler),
            Self::ManSet(fixture_selector, channel_name, channel_value) => self.run_man_set(
                fixture_handler,
                fixture_selector,
                channel_name,
                *channel_value,
            ),
            Self::FixtureSelector(_) => Ok(ActionRunResult::new()),
        }
    }

    fn run_go_home_all(
        &self,
        fixture_handler: &mut FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        fixture_handler
            .home_all()
            .map_err(ActionRunError::FixtureHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_go_home(
        &self,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector.get_fixtures();
        for fixture_id in fixtures {
            if let Some(fixture) = fixture_handler.fixture(fixture_id) {
                fixture.home().map_err(ActionRunError::FixtureError)?;
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_set_intensity(
        &self,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        intensity: u8,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector.get_fixtures();

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                let intens = f.intensity_ref().map_err(ActionRunError::FixtureError)?;
                *intens = Some(intensity);
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_man_set(
        &self,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        channel_name: &str,
        channel_value: u8,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector.get_fixtures();

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                let maintanence = f
                    .maintenance_ref(channel_name)
                    .map_err(ActionRunError::FixtureError)?;
                *maintanence = Some(channel_value);
            }
        }

        Ok(ActionRunResult::new())
    }
}
