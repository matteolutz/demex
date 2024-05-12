use crate::fixture::{handler::FixtureHandler, state::FixtureState};

use self::{error::ActionRunError, result::ActionRunResult};

use super::fixture_selector::FixtureSelector;

pub mod error;
pub mod result;

#[derive(Debug)]
pub enum Action {
    SetIntensity(FixtureSelector, u8),
    GoHome(FixtureSelector),
    GoHomeAll,
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
        }
    }

    fn run_go_home_all(
        &self,
        fixture_handler: &mut FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        fixture_handler
            .go_home_all()
            .map_err(ActionRunError::FixtureHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_go_home(
        &self,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector.get_fixtures();
        for fixture in fixtures {
            fixture_handler
                .go_home(fixture)
                .map_err(ActionRunError::FixtureHandlerError)?;
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
        println!("{:?}", fixtures);
        for fixture in fixtures {
            fixture_handler
                .set_fixture_state(fixture, FixtureState::from_intensity(intensity))
                .map_err(ActionRunError::FixtureHandlerError)?;
        }

        Ok(ActionRunResult::new())
    }
}
