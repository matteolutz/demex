use std::{collections::HashMap, f32, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        channel_value::{FixtureChannelValue2PresetState, FixtureChannelValue3},
        channel_value_discrete::FixtureChannelDiscreteValue,
        feature::{
            feature_group::FixtureChannel3FeatureGroup, feature_type::FixtureChannel3FeatureType,
        },
    },
    command::parser::nodes::{
        action::{ValueOrRange, functions::update_function::UpdateMode},
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
    effect::{feature::runtime::FeatureEffectRuntime, speed::EffectSpeed},
    fixture::GdtfFixturePatch,
    implement_set_property,
    keyframe_effect::{
        effect::KeyframeEffect, effect_keyframe::KeyframeEffectKeyframe,
        effect_keyframe_curve::KeyframeEffectKeyframeCurve, effect_runtime::KeyframeEffectRuntime,
    },
    patch::Patch,
    selection::FixtureSelection,
    state::{fixture_state::FixtureState, fixture_state_handler::FixtureStateHandler},
    timing::TimingHandler,
    updatables::runtime::RuntimePhase,
};

use super::{PresetHandler, error::PresetHandlerError};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default)]
pub struct FixturePresetId {
    pub feature_group: FixtureChannel3FeatureGroup,
    pub preset_id: u32,
}

impl ValueOrRange<FixturePresetId> {
    pub fn try_into_id_list(self) -> Result<Vec<FixturePresetId>, PresetHandlerError> {
        match self {
            ValueOrRange::Single(value) => Ok(vec![value]),
            ValueOrRange::Thru(from, to) => {
                if from.feature_group != to.feature_group {
                    return Err(PresetHandlerError::FeatureGroupMismatch(
                        from.feature_group,
                        to.feature_group,
                    ));
                }

                Ok((from.preset_id..=to.preset_id)
                    .map(|preset_id| FixturePresetId {
                        feature_group: from.feature_group,
                        preset_id,
                    })
                    .collect())
            }
        }
    }
}

impl PartialOrd for FixturePresetId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FixturePresetId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ord = self.feature_group.cmp(&other.feature_group);
        if ord == std::cmp::Ordering::Equal {
            self.preset_id.cmp(&other.preset_id)
        } else {
            ord
        }
    }
}

impl std::fmt::Display for FixturePresetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.feature_group.name(), self.preset_id)
    }
}

impl Serialize for FixturePresetId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self)
    }
}

impl<'de> Deserialize<'de> for FixturePresetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom("Invalid FixturePresetId"));
        }
        let feature_group = parts[0]
            .parse()
            .map_err(|_| serde::de::Error::custom("Failed to deserialize feature group"))?;
        let preset_id = parts[1].parse().map_err(serde::de::Error::custom)?;
        Ok(Self {
            feature_group,
            preset_id,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FixturePresetTarget {
    AllSelected,
    SomeSelected,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FixturePresetData {
    Default {
        data: HashMap<u32, HashMap<String, FixtureChannelDiscreteValue>>,
    },
    FeatureEffect {
        runtime: FeatureEffectRuntime,
    },
    KeyframeEffect {
        runtime: KeyframeEffectRuntime,
    },
}

impl Default for FixturePresetData {
    fn default() -> Self {
        Self::Default {
            data: HashMap::new(),
        }
    }
}

impl FixturePresetData {
    pub fn is_effect(&self) -> bool {
        matches!(
            self,
            Self::FeatureEffect { .. } | Self::KeyframeEffect { .. }
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixturePreset {
    id: FixturePresetId,

    name: String,

    #[serde(default)]
    display_color: Option<ecolor::Color32>,

    #[serde(default)]
    fade_up: f32,

    data: FixturePresetData,
}

impl FixturePreset {
    pub fn generate_preset_data(
        patch: &Patch,
        fixture_handler: &FixtureStateHandler,
        preset_handler: &mut PresetHandler,
        _timing_handler: &TimingHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        feature_group: FixtureChannel3FeatureGroup,
    ) -> Result<HashMap<u32, HashMap<String, FixtureChannelValue3>>, PresetHandlerError> {
        let mut data: HashMap<u32, HashMap<String, FixtureChannelValue3>> = HashMap::new();

        for fixture_id in fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?
            .fixtures()
        {
            let fixture = patch.fixture(*fixture_id);

            let Ok(fixture) = fixture else {
                continue;
            };

            let mut new_values = HashMap::new();

            let (fixture_type, dmx_mode) = patch.fixture_type_and_dmx_mode(fixture).unwrap();

            for dmx_channel in &dmx_mode.dmx_channels {
                // let channel = dmx_channel.logical_channels[0];

                // check, if the channel attribute belongs into the correct feature group
                // if not, skip it (continue)
                if feature_group != FixtureChannel3FeatureGroup::All
                    && dmx_channel.logical_channels[0]
                        .attribute(fixture_type)
                        .and_then(|attribute| {
                            attribute.feature(&fixture_type.attribute_definitions)
                        })
                        .and_then(|feature| {
                            FixtureChannel3FeatureType::from_str(
                                feature.name.as_ref().unwrap().as_ref(),
                            )
                            .ok()
                        })
                        .is_none_or(|feature| feature.feature_group() != feature_group)
                {
                    continue;
                }

                let value = fixture_handler
                    .fixture(fixture.id())
                    .unwrap()
                    .get_programmer_value(dmx_channel.name().as_ref());

                if let Ok(value) = value {
                    if value.is_home() {
                        continue;
                    }

                    new_values.insert(dmx_channel.name().as_ref().to_owned(), value.clone());
                }
            }

            // if we have values for this fixture, insert them
            if !new_values.is_empty() {
                data.insert(*fixture_id, new_values);
            }
        }

        Ok(data)
    }

    pub fn new(
        id: FixturePresetId,
        name: Option<String>,
        data: FixturePresetData,
    ) -> Result<Self, PresetHandlerError> {
        let name = name.unwrap_or(format!("Preset {}", id));

        Ok(Self {
            id,
            name,
            data,
            display_color: None,
            fade_up: 0.0,
        })
    }

    pub fn is_effect(&self) -> bool {
        self.data.is_effect()
    }

    pub fn data(&self) -> &FixturePresetData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut FixturePresetData {
        &mut self.data
    }

    pub fn affected_channels(&self, fixture: &GdtfFixturePatch, patch: &Patch) -> Vec<String> {
        match &self.data {
            FixturePresetData::Default { data } => data
                .get(&fixture.id())
                .map(|values| values.keys().cloned().collect())
                .unwrap_or_default(),
            FixturePresetData::FeatureEffect { runtime } => runtime
                .effect()
                .attributes()
                .flat_map(|attribute| fixture.channels_for_attribute(patch, attribute))
                .flatten()
                .map(|(channel, _, _)| channel.name().as_ref().to_string())
                .collect(),
            FixturePresetData::KeyframeEffect { runtime } => runtime
                .effect()
                .affected_channels_for_fixture(fixture.id())
                .iter()
                .map(|channel| channel.to_string())
                .collect(),
        }
    }

    pub fn apply(
        &self,
        patch: &Patch,
        fixture_id: u32,
        fixture_state: &mut FixtureState,
        new_selection: FixtureSelection,
    ) -> Result<(), PresetHandlerError> {
        let fixture = patch.fixture(fixture_id).unwrap();

        match &self.data {
            FixturePresetData::Default { data } => {
                if let Some(fixture_data) = data.get(&fixture.id()) {
                    for (preset_chanel_type, _) in fixture_data.iter() {
                        fixture_state
                            .set_programmer_value(
                                patch,
                                fixture,
                                preset_chanel_type.as_str(),
                                FixtureChannelValue3::Preset {
                                    id: self.id,
                                    state: Some(FixtureChannelValue2PresetState::now(
                                        new_selection.clone(),
                                    )),
                                },
                            )
                            .map_err(PresetHandlerError::FixtureError)?;
                    }
                }
            }
            FixturePresetData::FeatureEffect { runtime } => {
                for attribute in runtime.effect().attributes() {
                    // if the fixture doesn't have this feature type, skip
                    if let Ok(channels) = fixture.channels_for_attribute(patch, attribute) {
                        for (dmx_channel, _, _) in channels {
                            fixture_state
                                .set_programmer_value(
                                    patch,
                                    fixture,
                                    dmx_channel.name().as_ref(),
                                    FixtureChannelValue3::Preset {
                                        id: self.id,
                                        state: Some(FixtureChannelValue2PresetState::now(
                                            new_selection.clone(),
                                        )),
                                    },
                                )
                                .map_err(PresetHandlerError::FixtureError)?;
                        }
                    }
                }
            }
            FixturePresetData::KeyframeEffect { runtime } => {
                for channel in runtime.effect().affected_channels_for_fixture(fixture.id()) {
                    fixture_state
                        .set_programmer_value(
                            patch,
                            fixture,
                            channel,
                            FixtureChannelValue3::Preset {
                                id: self.id,
                                state: Some(FixtureChannelValue2PresetState::now(
                                    new_selection.clone(),
                                )),
                            },
                        )
                        .map_err(PresetHandlerError::FixtureError)?;
                }
            }
        }

        Ok(())
    }

    pub fn get_target(&self, selected_fixtures: &[u32]) -> FixturePresetTarget {
        match &self.data {
            FixturePresetData::Default { data } => {
                let mutual = data
                    .keys()
                    .filter(|k| selected_fixtures.contains(k))
                    .collect::<Vec<_>>();

                if mutual.is_empty() {
                    FixturePresetTarget::None
                } else if mutual.len() == selected_fixtures.len() {
                    FixturePresetTarget::AllSelected
                } else {
                    FixturePresetTarget::SomeSelected
                }
            }
            FixturePresetData::FeatureEffect { .. } => FixturePresetTarget::AllSelected,
            FixturePresetData::KeyframeEffect { runtime } => {
                let affected_fixtures = runtime
                    .effect()
                    .affected_fixtures()
                    .iter()
                    .filter(|fixture_id| selected_fixtures.contains(fixture_id))
                    .count();

                if affected_fixtures == 0 {
                    FixturePresetTarget::None
                } else if affected_fixtures == selected_fixtures.len() {
                    FixturePresetTarget::AllSelected
                } else {
                    FixturePresetTarget::SomeSelected
                }
            }
        }
    }

    pub fn id(&self) -> FixturePresetId {
        self.id
    }

    pub fn move_to(&mut self, id: FixturePresetId) {
        self.id = id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn display_color(&self) -> Option<ecolor::Color32> {
        self.display_color
    }

    pub fn display_color_mut(&mut self) -> &mut Option<ecolor::Color32> {
        &mut self.display_color
    }

    pub fn values(
        &self,
        patch: &Patch,
        fixture: &GdtfFixturePatch,
        _preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        state: Option<&FixtureChannelValue2PresetState>,
    ) -> Vec<(String, FixtureChannelValue3)> {
        match &self.data {
            FixturePresetData::Default { data } => data
                .get(&fixture.id())
                .map(|values| {
                    values
                        .iter()
                        .map(|(channel_name, value)| {
                            (
                                channel_name.clone(),
                                FixtureChannelValue3::Discrete(value.clone()),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),

            // TODO: rework this
            FixturePresetData::FeatureEffect { runtime } => {
                let fixture_offset = state
                    .and_then(|state| state.selection().offset(fixture.id()))
                    .unwrap_or_default();

                runtime.get_values_with_started(
                    patch,
                    fixture,
                    fixture_offset,
                    timing_handler,
                    state.map(|state| state.started()),
                )
            }
            FixturePresetData::KeyframeEffect { runtime } => {
                let fixture_offset = state
                    .and_then(|state| state.selection().offset(fixture.id()))
                    .unwrap_or_default();

                let channels = runtime.effect().affected_channels_for_fixture(fixture.id());

                channels
                    .iter()
                    .map(|channel| {
                        (
                            channel.to_string(),
                            runtime
                                .get_channel_value_with_started(
                                    channel,
                                    fixture,
                                    fixture_offset,
                                    timing_handler,
                                    state.map(|state| state.started()),
                                )
                                .unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
            }
        }
    }

    pub fn value(
        &self,
        patch: &Patch,
        fixture: &GdtfFixturePatch,
        channel_name: &str,
        _preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        state: Option<&FixtureChannelValue2PresetState>,
    ) -> Option<FixtureChannelValue3> {
        let started_delta = state
            .map(|state| state.started().elapsed().as_secs_f32())
            .unwrap_or(0.0);

        let _fade = if self.fade_up > 0.0 {
            (started_delta / self.fade_up).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let val = match &self.data {
            FixturePresetData::Default { data } => data
                .get(&fixture.id())
                .and_then(|values| values.get(channel_name).cloned())
                .map(FixtureChannelValue3::Discrete),
            FixturePresetData::FeatureEffect { runtime } => {
                let fixture_offset = state
                    .and_then(|state| state.selection().offset(fixture.id()))
                    .unwrap_or_default();

                runtime
                    .get_channel_value_with_started(
                        channel_name,
                        patch,
                        fixture,
                        fixture_offset,
                        timing_handler,
                        state.map(|state| state.started()),
                    )
                    .ok()
            }
            FixturePresetData::KeyframeEffect { runtime } => {
                let fixture_offset = state
                    .and_then(|state| state.selection().offset(fixture.id()))
                    .unwrap_or_default();

                runtime
                    .get_channel_value_with_started(
                        channel_name,
                        fixture,
                        fixture_offset,
                        timing_handler,
                        state.map(|state| state.started()),
                    )
                    .ok()
            }
        };

        // val.map(|val| ((val as f32 / 255.0) * fade * 255.0) as u8)
        val
    }

    pub fn record_next(
        &mut self,
        data: HashMap<u32, HashMap<String, FixtureChannelValue3>>,
    ) -> Result<(), PresetHandlerError> {
        match &mut self.data {
            FixturePresetData::FeatureEffect { .. } => {
                Err(PresetHandlerError::PresetCannotRecordNextKeyframe(self.id))
            }
            FixturePresetData::KeyframeEffect { runtime } => {
                runtime.effect_mut().layers_mut()[0].add_keyframe(KeyframeEffectKeyframe::new(
                    0.0,
                    data,
                    KeyframeEffectKeyframeCurve::default(),
                ));

                Ok(())
            }
            FixturePresetData::Default { data } => {
                let effect_runtime = KeyframeEffectRuntime::new(
                    KeyframeEffect::form_data(data.clone()),
                    EffectSpeed::default(),
                    RuntimePhase::default(),
                );
                self.data = FixturePresetData::KeyframeEffect {
                    runtime: effect_runtime,
                };

                Ok(())
            }
        }
    }

    pub fn update(
        &mut self,
        values_to_update: HashMap<u32, HashMap<String, FixtureChannelDiscreteValue>>,
        update_mode: UpdateMode,
    ) -> Result<usize, PresetHandlerError> {
        match &mut self.data {
            FixturePresetData::Default { data } => {
                let mut updated = 0;

                for (fixture_id, new_fixture_values) in values_to_update {
                    // if we already have a value for this fixture and we are not in override mode, skip
                    if data.contains_key(&fixture_id) && update_mode != UpdateMode::Override {
                        continue;
                    }

                    // Insert or update
                    data.insert(fixture_id, new_fixture_values);

                    updated += 1;
                }

                Ok(updated)
            }
            FixturePresetData::FeatureEffect { .. } => Ok(0),
            FixturePresetData::KeyframeEffect { .. } => Ok(0),
        }
    }
}

#[derive(strum_macros::EnumString, strum_macros::Display)]
pub enum FixturePresetProperty {
    Name,
}

implement_set_property! {
    for FixturePreset with FixturePresetProperty,

    Name => name as String
}
