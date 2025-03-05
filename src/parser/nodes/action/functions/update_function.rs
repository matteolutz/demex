use serde::{Deserialize, Serialize};

use crate::{
    fixture::{presets::preset::FixturePresetId, sequence::cue::CueIdx, timing::TimingHandler},
    parser::nodes::{
        action::{error::ActionRunError, result::ActionRunResult},
        fixture_selector::FixtureSelector,
    },
};

use super::{record_function::RecordChannelTypeSelector, FunctionArgs};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateMode {
    Merge,
    Override,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePresetArgs {
    pub id: FixturePresetId,
    pub fixture_selector: FixtureSelector,
    pub update_mode: UpdateMode,
}

impl FunctionArgs for UpdatePresetArgs {
    fn run(
        &self,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        timing_handler: &mut TimingHandler,
    ) -> Result<
        crate::parser::nodes::action::result::ActionRunResult,
        crate::parser::nodes::action::error::ActionRunError,
    > {
        let num_updated = preset_handler
            .update_preset(
                &self.fixture_selector,
                fixture_selector_context,
                self.id,
                fixture_handler,
                timing_handler,
                self.update_mode,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        if num_updated == 0 {
            Ok(ActionRunResult::Warn(
                "No fixtures we're affected. If you're trying override existing preset data, try running with the \"override\" flag.".to_owned(),
            ))
        } else {
            Ok(ActionRunResult::new())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSequenceCueArgs {
    pub sequence_id: u32,
    pub cue_idx: CueIdx,
    pub fixture_selector: FixtureSelector,
    pub channel_type_selector: RecordChannelTypeSelector,
    pub update_mode: UpdateMode,
}

impl FunctionArgs for UpdateSequenceCueArgs {
    fn run(
        &self,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        let num_updated = preset_handler
            .update_sequence_cue(
                self.sequence_id,
                self.cue_idx,
                &self.fixture_selector,
                fixture_selector_context,
                fixture_handler,
                &self.channel_type_selector,
                self.update_mode,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        if num_updated == 0 {
            Ok(ActionRunResult::Warn(
                "No fixtures we're affected. If you're trying override existing preset data, try running with the \"override\" flag.".to_owned(),
            ))
        } else {
            Ok(ActionRunResult::new())
        }
    }
}
