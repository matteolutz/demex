use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel3::feature::feature_type::FixtureChannel3FeatureType,
        patch::Patch,
        presets::{preset::FixturePresetId, PresetHandler},
        selection::FixtureSelection,
        timing::TimingHandler,
    },
    parser::nodes::{
        action::{error::ActionRunError, result::ActionRunResult, ValueOrRange},
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
};

use super::FunctionArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFeatureValueArgs {
    pub fixture_selector: FixtureSelector,
    pub feature_type: FixtureChannel3FeatureType,
    pub feature_value: ValueOrRange<f32>,
}

impl FunctionArgs for SetFeatureValueArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<
        crate::parser::nodes::action::result::ActionRunResult,
        crate::parser::nodes::action::error::ActionRunError,
    > {
        let selection = self
            .fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for fixture_id in selection.fixtures() {
            let fixture_idx = selection.offset_idx(*fixture_id).unwrap();

            let _discrete_value = match self.feature_value {
                ValueOrRange::Single(value) => value,
                ValueOrRange::Thru(start, end) => {
                    let range = end - start;
                    let step = range / (selection.num_offsets() - 1) as f32;
                    start + step * fixture_idx as f32
                }
            };

            if let Some(_fixture) = fixture_handler.fixture(*fixture_id) {
                todo!();
            }
        }

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionOrSelector {
    Selection(FixtureSelection),
    Selector(FixtureSelector),
}

impl SelectionOrSelector {
    pub fn get_selection(
        &self,
        preset_handler: &PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<FixtureSelection, ActionRunError> {
        match self {
            Self::Selection(selection) => Ok(selection.clone()),
            Self::Selector(selector) => selector
                .get_selection(preset_handler, fixture_selector_context)
                .map_err(ActionRunError::FixtureSelectorError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFixturePresetArgs {
    pub selection_or_selector: SelectionOrSelector,
    pub preset_id: ValueOrRange<FixturePresetId>,
}

impl FunctionArgs for SetFixturePresetArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        let selection = self
            .selection_or_selector
            .get_selection(preset_handler, fixture_selector_context)?;

        match self.preset_id {
            ValueOrRange::Single(preset_id) => {
                preset_handler
                    .apply_preset(preset_id, fixture_handler, patch.fixture_types(), selection)
                    .map_err(ActionRunError::PresetHandlerError)?;
            }
            ValueOrRange::Thru(_, _) => {
                todo!()
                /*let presets = preset_handler
                    .get_preset_range(preset_id_from, preset_id_to, channel_type)
                    .map_err(ActionRunError::PresetHandlerError)?;

                for (idx, fixture) in fixtures.iter().enumerate() {
                    if let Some(f) = fixture_handler.fixture(*fixture) {
                        // get the two relevant indexes from the presets
                        let preset_idx_fl =
                            idx as f32 * ((presets.len() - 1) as f32 / fixtures.len() as f32);

                        let preset_idx_low = preset_idx_fl.floor() as usize;
                        let preset_idx_high = preset_idx_low + 1;

                        let fade = (idx as f32 * ((presets.len()) as f32 / fixtures.len() as f32))
                            - preset_idx_low as f32;

                        let channel_value = FixtureChannelValue::Mix {
                            a: Box::new(FixtureChannelValue::Preset(presets[preset_idx_low].id())),
                            b: Box::new(FixtureChannelValue::Preset(presets[preset_idx_high].id())),
                            mix: fade,
                        };

                        f.set_channel_value(channel_type, channel_value)
                            .map_err(ActionRunError::FixtureError)?;
                    }
                }
                */
            }
        }

        Ok(ActionRunResult::new())
    }
}
