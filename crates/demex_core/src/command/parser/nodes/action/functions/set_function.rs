use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute,
        channel_value::{FixtureChannelValue2PresetState, FixtureChannelValue3},
    },
    command::parser::nodes::{
        action::{ActionRunArgs, ValueOrRange, error::ActionRunError, result::ActionRunResult},
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
        object::{Object, ObjectDelegate},
    },
    event::DemexEvent,
    presets::{PresetHandler, preset::FixturePresetId},
    selection::FixtureSelection,
    sequence::cue::{CueIdx, CueProperty, CueTrigger},
};

use super::FunctionDelegate;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SetAttributeValue {
    Absolute(ValueOrRange<f32>),
    RelativChange(ValueOrRange<f32>),
}

impl SetAttributeValue {
    pub fn value_and_relative(self) -> (ValueOrRange<f32>, bool) {
        match self {
            SetAttributeValue::Absolute(value) => (value, false),
            SetAttributeValue::RelativChange(value) => (value, true),
        }
    }
}

impl<T> From<T> for SetAttributeValue
where
    T: Into<ValueOrRange<f32>>,
{
    fn from(value: T) -> Self {
        SetAttributeValue::Absolute(value.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAttributeValueArgs {
    pub fixture_selector: FixtureSelector,
    pub attribute: FixtureChannel3Attribute,
    pub attribute_value: Option<SetAttributeValue>,
}

impl FunctionDelegate for SetAttributeValueArgs {
    fn run(
        &self,
        args: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        let selection = self
            .fixture_selector
            .get_selection(args.preset_handler, args.fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for fixture_path in selection.fixtures() {
            let fixture_idx = selection.offset_idx(fixture_path).unwrap();

            let value = match self.attribute_value {
                Some(value) => {
                    let (value, is_relative) = value.value_and_relative();

                    let mut f_value = match value {
                        ValueOrRange::Single(value) => value,
                        ValueOrRange::Thru(start, end) => {
                            let range = end - start;
                            let step = range / (selection.num_offsets() - 1) as f32;
                            start + step * fixture_idx as f32
                        }
                    };

                    // if the set mode is relative we want to..
                    if is_relative {
                        // ...find the fixture...
                        let fixture = args.patch.fixture(fixture_path).unwrap();

                        // ...get its current value and the channel function
                        // for the attribute we want to set...
                        if let Some((current_value, cf)) = args
                            .fixture_handler
                            .fixture(fixture_path)
                            .and_then(|state| state.get_programmer_value(&self.attribute))
                            .cloned()
                            .ok()
                            .zip(fixture.channel_function(&self.attribute))
                        {
                            // ...discretize the current value (for presets this means
                            // getting the current value form the preset)...
                            let discrete_value = current_value.to_discrete(
                                fixture,
                                &self.attribute,
                                args.preset_handler,
                                args.timing_handler,
                            );

                            // ...convert it to a clamped value using the channel function...
                            let clamped_value = discrete_value.to_clamped(cf);

                            // ...and add it to the value we want to set
                            f_value += clamped_value.as_f32();
                        }
                    }

                    FixtureChannelValue3::discrete(f_value)
                }
                None => FixtureChannelValue3::home(),
            };

            if let (Ok(fixture_state), Ok(fixture)) = (
                args.fixture_handler.fixture_mut(fixture_path),
                args.patch.fixture(fixture_path),
            ) {
                fixture_state
                    .set_programmer_value(fixture, &self.attribute, value)
                    .map_err(ActionRunError::FixtureError)?;
            }
        }

        args.event_list.push(DemexEvent::FixtureValuesChanged(
            selection.fixtures().to_vec(),
        ));
        Ok(ActionRunResult::Default)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAttributeChannelSetArgs {
    pub fixture_selector: FixtureSelector,
    pub attribute: FixtureChannel3Attribute,
    pub channel_set: String,
}

impl FunctionDelegate for SetAttributeChannelSetArgs {
    fn run(
        &self,
        args: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        let selection = self
            .fixture_selector
            .get_selection(args.preset_handler, args.fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for fixture in selection
            .fixtures()
            .iter()
            .filter_map(|f_path| args.patch.fixture(f_path).ok())
            .filter(|fixture| {
                fixture
                    .channel_function(&self.attribute)
                    .is_some_and(|cf| cf.has_channel_set(&self.channel_set))
            })
        {
            if let Ok(fixture_state) = args.fixture_handler.fixture_mut(&fixture.path()) {
                fixture_state
                    .set_programmer_value(
                        fixture,
                        &self.attribute,
                        FixtureChannelValue3::discrete_set(self.channel_set.clone()),
                    )
                    .map_err(ActionRunError::FixtureError)?;
            }
        }

        args.event_list.push(DemexEvent::FixtureValuesChanged(
            selection.fixtures().to_vec(),
        ));
        Ok(ActionRunResult::Default)
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

impl FunctionDelegate for SetFixturePresetArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        let selection = self
            .selection_or_selector
            .get_selection(args.preset_handler, args.fixture_selector_context)?;

        let fixtures = selection.fixtures().to_vec();

        match self.preset_id {
            ValueOrRange::Single(preset_id) => {
                args.preset_handler
                    .apply_preset(preset_id, args.fixture_handler, args.patch, selection)
                    .map_err(ActionRunError::PresetHandlerError)?;
            }
            ValueOrRange::Thru(preset_id_from, preset_id_to) => {
                let presets = args
                    .preset_handler
                    .get_preset_range(preset_id_from, preset_id_to)
                    .map_err(ActionRunError::PresetHandlerError)?;

                for fixture_path in selection.fixtures().iter() {
                    let fixture_offset = selection.offset(fixture_path).unwrap();

                    if let (Ok(state), Ok(fixture)) = (
                        args.fixture_handler.fixture_mut(fixture_path),
                        args.patch.fixture(fixture_path),
                    ) {
                        let attributes = presets[0].stored_attributes(fixture_path);

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

                        for attribute in attributes {
                            state
                                .set_programmer_value(fixture, &attribute, channel_value.clone())
                                .map_err(ActionRunError::FixtureError)?;
                        }
                    }
                }
            }
        }

        args.event_list
            .push(DemexEvent::FixtureValuesChanged(fixtures));
        Ok(ActionRunResult::Default)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSetPropertyArgs {
    pub object: Object,
    pub key: String,
    pub value: String,
}

impl FunctionDelegate for ObjectSetPropertyArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        self.object.clone().set(
            args.preset_handler,
            args.updatable_handler,
            args.fixture_selector_context,
            args.event_list,
            self.key.clone(),
            self.value.clone(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CueSetTriggerArgs {
    pub sequence_id: u32,
    pub cue_idx: CueIdx,
    pub trigger: CueTrigger,
}

impl FunctionDelegate for CueSetTriggerArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        Object::SequenceCue(self.sequence_id, self.cue_idx).set_any(
            args.preset_handler,
            args.updatable_handler,
            args.fixture_selector_context,
            args.event_list,
            CueProperty::Trigger.to_string(),
            Box::new(self.trigger.clone()),
        )
    }
}
