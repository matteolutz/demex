use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel2::feature::feature_type::FixtureFeatureType,
        error::FixtureError,
        presets::preset::FixturePresetId,
        sequence::cue::{CueFixtureChannelValue, CueIdx},
        Fixture,
    },
    parser::nodes::{
        action::{error::ActionRunError, result::ActionRunResult},
        fixture_selector::FixtureSelector,
    },
};

use super::FunctionArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordChannelTypeSelector {
    All,
    Active,
    Features(Vec<FixtureFeatureType>),
}

impl RecordChannelTypeSelector {
    pub fn get_channel_values(
        &self,
        fixture: &Fixture,
    ) -> Result<Vec<CueFixtureChannelValue>, FixtureError> {
        let mut values = Vec::new();

        for channel_type in fixture.channel_types() {
            match self {
                Self::All => {
                    values.push(CueFixtureChannelValue::new(
                        fixture.channel_value_programmer(*channel_type)?.clone(),
                        *channel_type,
                        false,
                    ));
                }
                Self::Active => {
                    let value = fixture.channel_value_programmer(*channel_type)?;
                    if value.is_home() {
                        continue;
                    }

                    values.push(CueFixtureChannelValue::new(
                        value.clone(),
                        *channel_type,
                        false,
                    ));
                }
                Self::Features(_channels) => {
                    todo!()
                }
            }
        }

        Ok(values)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPresetArgs {
    pub id: FixturePresetId,
    pub fixture_selector: FixtureSelector,
    pub name: Option<String>,
}

impl FunctionArgs for RecordPresetArgs {
    fn run(
        &self,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
    ) -> Result<
        crate::parser::nodes::action::result::ActionRunResult,
        crate::parser::nodes::action::error::ActionRunError,
    > {
        preset_handler
            .record_preset(
                &self.fixture_selector,
                fixture_selector_context,
                self.id,
                self.name.clone(),
                fixture_handler,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordGroupArgs {
    pub id: Option<u32>,
    pub fixture_selector: FixtureSelector,
    pub name: Option<String>,
}

impl FunctionArgs for RecordGroupArgs {
    fn run(
        &self,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        let selection = self
            .fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        preset_handler
            .record_group(
                selection,
                self.id.unwrap_or_else(|| preset_handler.next_group_id()),
                self.name.clone(),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordSequenceCueArgs {
    pub sequence_id: u32,
    pub cue_idx: Option<CueIdx>,
    pub fixture_selector: FixtureSelector,
    pub channel_type_selector: RecordChannelTypeSelector,
}

impl FunctionArgs for RecordSequenceCueArgs {
    fn run(
        &self,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_sequence_cue(
                self.sequence_id,
                fixture_handler,
                &self.fixture_selector,
                fixture_selector_context,
                self.cue_idx,
                &self.channel_type_selector,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}
