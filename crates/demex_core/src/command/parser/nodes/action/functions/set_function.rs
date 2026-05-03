use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute,
        channel_value::{FixtureChannelValue2PresetState, FixtureChannelValue3},
    },
    color::color_space::RgbValue,
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SetFeatureValue {
    Rgb { value: RgbValue, use_white: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFeatureValueArgs {
    pub fixture_selector: FixtureSelector,
    pub feature: SetFeatureValue,
}

impl FunctionDelegate for SetFeatureValueArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        let selection = self
            .fixture_selector
            .get_selection(args.preset_handler, args.fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        match self.feature {
            SetFeatureValue::Rgb { value, use_white } => {
                for f_path in selection.fixtures() {
                    // copy the color for each fixture so we can modify it
                    let mut color = value;

                    let Some((fixture, fixture_state)) = args
                        .patch
                        .fixture(f_path)
                        .ok()
                        .zip(args.fixture_handler.fixture_mut(f_path).ok())
                    else {
                        continue;
                    };

                    // TODO: handle more color attributes

                    // if the fixture has a white channel and `use_white` is true
                    // extract a white component from the rgb values and use that
                    if use_white {
                        if fixture.has_attribute(&FixtureChannel3Attribute::ColorAddW) {
                            let white = color.extract_white();
                            let _ = fixture_state.set_programmer_value(
                                fixture,
                                &FixtureChannel3Attribute::ColorAddW,
                                FixtureChannelValue3::discrete(white),
                            );
                        // we are using `else if`, because we only really want to use one white channel
                        // this will be maybe replaced by a more elaborate system in the future
                        } else if fixture.has_attribute(&FixtureChannel3Attribute::ColorAddWW) {
                            let white = color.extract_warm_white();
                            let _ = fixture_state.set_programmer_value(
                                fixture,
                                &FixtureChannel3Attribute::ColorAddWW,
                                FixtureChannelValue3::discrete(white),
                            );
                        }
                    }

                    // secondary emitters
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorAddRY) {
                        let amber = color.extract_amber();
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorAddRY,
                            FixtureChannelValue3::discrete(amber),
                        );
                    }
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorAddC) {
                        let cyan = color.extract_cyan();
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorAddC,
                            FixtureChannelValue3::discrete(cyan),
                        );
                    }
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorAddM) {
                        let magenta = color.extract_magenta();
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorAddM,
                            FixtureChannelValue3::discrete(magenta),
                        );
                    }
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorAddY) {
                        let yellow = color.extract_yellow();
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorAddY,
                            FixtureChannelValue3::discrete(yellow),
                        );
                    }

                    // primary additive emitters
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorAddR) {
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorAddR,
                            FixtureChannelValue3::discrete(color.r),
                        );
                    }
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorAddG) {
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorAddG,
                            FixtureChannelValue3::discrete(color.g),
                        );
                    }
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorAddB) {
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorAddB,
                            FixtureChannelValue3::discrete(color.b),
                        );
                    }

                    // subtractive emitters
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorSubC) {
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorSubC,
                            FixtureChannelValue3::discrete(1.0 - color.r),
                        );
                    }
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorSubM) {
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorSubM,
                            FixtureChannelValue3::discrete(1.0 - color.g),
                        );
                    }
                    if fixture.has_attribute(&FixtureChannel3Attribute::ColorSubY) {
                        let _ = fixture_state.set_programmer_value(
                            fixture,
                            &FixtureChannel3Attribute::ColorSubY,
                            FixtureChannelValue3::discrete(1.0 - color.b),
                        );
                    }
                }
            }
        }

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
