use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute,
        channel_value::{
            FixtureChannelValue2PresetState, FixtureChannelValue3, FixtureChannelValue3Update,
        },
        feature::feature_type::FixtureChannel3FeatureType,
    },
    command::parser::nodes::{
        action::{ValueOrRange, error::ActionRunError, result::ActionRunResult},
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
        object::{Object, ObjectDelegate},
    },
    event::DemexEvent,
    patch::Patch,
    presets::{PresetHandler, preset::FixturePresetId},
    selection::FixtureSelection,
    timing::TimingHandler,
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
        fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        preset_handler: &mut crate::presets::PresetHandler,
        fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        let selection = self
            .fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for fixture_id in selection.fixtures() {
            let fixture_idx = selection.offset_idx(*fixture_id).unwrap();

            let discrete_value = match self.feature_value {
                ValueOrRange::Single(value) => value,
                ValueOrRange::Thru(start, end) => {
                    let range = end - start;
                    let step = range / (selection.num_offsets() - 1) as f32;
                    start + step * fixture_idx as f32
                }
            };

            if let (Ok(fixture_state), Ok(fixture)) = (
                fixture_handler.fixture_mut(*fixture_id),
                patch.fixture(*fixture_id),
            ) {
                match self.feature_type {
                    FixtureChannel3FeatureType::Dimmer => {
                        fixture_state
                            .update_programmer_attribute_matches_value(
                                patch,
                                fixture,
                                |fixture_attribute_name| {
                                    FixtureChannel3Attribute::attribute_matches(
                                        fixture_attribute_name,
                                        FixtureChannel3Attribute::Dimmer.to_string().as_str(),
                                    )
                                },
                                FixtureChannelValue3Update::Value(discrete_value),
                            )
                            .map_err(ActionRunError::FixtureError)?;
                    }
                    _ => todo!(),
                }
            }
        }

        Ok(ActionRunResult::event(DemexEvent::FixtureValuesChanged(
            selection.fixtures().to_vec(),
        )))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionOrSelector {
    Selection(FixtureSelection),
    Selector(FixtureSelector),
    Current,
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
            Self::Current => fixture_selector_context
                .current_fixture()
                .ok_or(ActionRunError::NoFixtureSelected)
                .cloned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFixturePresetArgs {
    pub selection_or_selector: SelectionOrSelector,
    pub preset_id: ValueOrRange<FixturePresetId>,
}

impl SetFixturePresetArgs {
    pub fn current(preset_id: FixturePresetId) -> Self {
        Self {
            selection_or_selector: SelectionOrSelector::Current,
            preset_id: ValueOrRange::Single(preset_id),
        }
    }
}

impl FunctionArgs for SetFixturePresetArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        preset_handler: &mut crate::presets::PresetHandler,
        fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        let selection = self
            .selection_or_selector
            .get_selection(preset_handler, fixture_selector_context)?;

        let fixtures = selection.fixtures().to_vec();

        match self.preset_id {
            ValueOrRange::Single(preset_id) => {
                preset_handler
                    .apply_preset(preset_id, fixture_handler, patch, selection)
                    .map_err(ActionRunError::PresetHandlerError)?;
            }
            ValueOrRange::Thru(preset_id_from, preset_id_to) => {
                let presets = preset_handler
                    .get_preset_range(preset_id_from, preset_id_to)
                    .map_err(ActionRunError::PresetHandlerError)?;

                for fixture in selection.fixtures().iter() {
                    let fixture_offset = selection.offset(*fixture).unwrap();

                    if let (Ok(state), Ok(fixture)) = (
                        fixture_handler.fixture_mut(*fixture),
                        patch.fixture(*fixture),
                    ) {
                        let channels = presets[0].affected_channels(fixture, patch);

                        // get the two relevant indexes from the presets
                        let preset_idx_fl = fixture_offset
                            * ((presets.len() - 1) as f32 / (selection.num_offsets() - 1) as f32);

                        let preset_idx_low = preset_idx_fl.floor() as usize;
                        let preset_idx_high = preset_idx_low + 1;

                        let fade = (fixture_offset
                            * ((presets.len()) as f32 / (selection.num_offsets() - 1) as f32))
                            - preset_idx_low as f32;

                        let channel_value = FixtureChannelValue3::Mix {
                            a: Box::new(FixtureChannelValue3::Preset {
                                id: presets[preset_idx_low].id(),
                                state: Some(FixtureChannelValue2PresetState::now(
                                    selection.clone(),
                                )),
                            }),
                            b: Box::new(FixtureChannelValue3::Preset {
                                id: presets[preset_idx_high].id(),
                                state: Some(FixtureChannelValue2PresetState::now(
                                    selection.clone(),
                                )),
                            }),
                            mix: fade,
                        };

                        for channel in channels {
                            state
                                .set_programmer_value(
                                    patch,
                                    fixture,
                                    &channel,
                                    channel_value.clone(),
                                )
                                .map_err(ActionRunError::FixtureError)?;
                        }
                    }
                }
            }
        }

        Ok(ActionRunResult::event(DemexEvent::FixtureValuesChanged(
            fixtures,
        )))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSetPropertyArgs {
    pub object: Object,
    pub key: String,
    pub value: String,
}

impl FunctionArgs for ObjectSetPropertyArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        _fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _timing_handler: &mut TimingHandler,
        _patch: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        self.object.clone().set(
            preset_handler,
            updatable_handler,
            fixture_selector_context,
            self.key.clone(),
            self.value.clone(),
        )
    }
}
