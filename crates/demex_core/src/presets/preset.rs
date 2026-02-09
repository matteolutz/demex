use std::{collections::HashMap, f32};

use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute,
        channel_value::{FixtureChannelValue2PresetState, FixtureChannelValue3},
        channel_value_discrete::FixtureChannelDiscreteValue,
        feature::feature_group::FixtureChannel3FeatureGroup,
    },
    command::parser::nodes::{
        action::{ValueOrRange, functions::update_function::UpdateMode},
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
    effect::{feature::runtime::FeatureEffectRuntime, speed::EffectSpeed},
    event::{DemexEvent, list::DemexEventList},
    fixture::{Fixture, FixturePath},
    implement_set_property,
    keyframe_effect::{
        effect::KeyframeEffect, effect_keyframe::KeyframeEffectKeyframe,
        effect_keyframe_curve::KeyframeEffectKeyframeCurve, effect_runtime::KeyframeEffectRuntime,
    },
    patch::Patch,
    pool::{PoolItem, PoolType},
    selection::FixtureSelection,
    state::{fixture_state::FixtureState, fixture_state_handler::FixtureStateHandler},
    timing::TimingHandler,
    updatables::runtime::RuntimePhase,
    utils::color::rgbw_to_rgb,
};

use super::{PresetHandler, error::PresetHandlerError};

pub(super) const MAX_DISPLAY_COLORS: usize = 5;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default)]
pub struct FixturePresetId {
    pub feature_group: FixtureChannel3FeatureGroup,
    pub preset_id: u32,
}

impl FixturePresetId {
    pub fn new(feature_group: FixtureChannel3FeatureGroup, preset_id: u32) -> Self {
        Self {
            feature_group,
            preset_id,
        }
    }
}

impl From<(FixtureChannel3FeatureGroup, u32)> for FixturePresetId {
    fn from((feature_group, preset_id): (FixtureChannel3FeatureGroup, u32)) -> Self {
        Self::new(feature_group, preset_id)
    }
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
        data: HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>>,
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

/// 12-bit RGB color (4bit per channel)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct FixturePresetDisplayColor(u16);

impl FixturePresetDisplayColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        let r_4 = ((r >> 4) & 0xF) as u16;
        let g_4 = ((g >> 4) & 0xF) as u16;
        let b_4 = ((b >> 4) & 0xF) as u16;
        Self((r_4 << 8) | (g_4 << 4) | b_4)
    }

    pub fn from_rgbw_f(rgbw: [f32; 4]) -> Self {
        let [r, g, b] = rgbw_to_rgb(rgbw);
        Self::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }

    pub fn from_rgbw([r, g, b, w]: [u8; 4]) -> Self {
        Self::from_rgbw_f([
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            w as f32 / 255.0,
        ])
    }

    pub fn r(self) -> u8 {
        let r_4 = ((self.0 >> 8) & 0xF) as u8;
        r_4 << 4
    }

    pub fn g(self) -> u8 {
        let g_4 = ((self.0 >> 4) & 0xF) as u8;
        g_4 << 4
    }

    pub fn b(self) -> u8 {
        let b_4 = (self.0 & 0xF) as u8;
        b_4 << 4
    }

    pub fn rgb(self) -> [u8; 3] {
        [self.r(), self.g(), self.b()]
    }

    pub fn rgb_f(self) -> [f32; 3] {
        [
            self.r() as f32 / 255.0,
            self.g() as f32 / 255.0,
            self.b() as f32 / 255.0,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixturePreset {
    id: FixturePresetId,

    name: String,

    #[serde(default)]
    display_colors: Vec<FixturePresetDisplayColor>,

    #[serde(default)]
    fade_up: f32,

    data: FixturePresetData,
}

impl FixturePreset {
    pub fn generate_preset_data(
        patch: &Patch,
        fixture_handler: &FixtureStateHandler,
        preset_handler: &mut PresetHandler,
        timing_handler: &TimingHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        feature_group: FixtureChannel3FeatureGroup,
    ) -> Result<
        HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>>,
        PresetHandlerError,
    > {
        let mut data: HashMap<
            FixturePath,
            HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>,
        > = HashMap::new();

        for fixture_path in fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?
            .fixtures()
        {
            let fixture = patch.fixture(fixture_path);
            log::debug!("Fixture {}", fixture_path);

            let Ok(fixture) = fixture else {
                continue;
            };

            let mut new_values = HashMap::new();

            for (attribute, _) in fixture.channel_functions() {
                // let channel = dmx_channel.logical_channels[0];

                // check, if the channel attribute belongs into the correct feature group
                // if not, skip it (continue)
                if feature_group != FixtureChannel3FeatureGroup::All
                    && attribute
                        .feature_type()
                        .is_none_or(|ft| ft.feature_group() != feature_group)
                {
                    continue;
                }

                log::debug!("Using attribute: {}", attribute);

                let value = fixture_handler
                    .fixture(fixture_path)
                    .unwrap()
                    .get_programmer_value(attribute);

                let Ok(value) = value else {
                    continue;
                };

                log::debug!("got value: {:?}", value);

                if value.is_home() {
                    continue;
                }

                new_values.insert(
                    *attribute,
                    value
                        .clone()
                        .to_discrete(fixture, attribute, preset_handler, timing_handler),
                );
            }

            // if we have values for this fixture, insert them
            if !new_values.is_empty() {
                data.insert(*fixture_path, new_values);
            }
        }

        log::debug!("data: {:?}", data);

        Ok(data)
    }

    pub fn new(
        id: FixturePresetId,
        name: Option<String>,
        data: FixturePresetData,
        mut display_colors: Vec<FixturePresetDisplayColor>,
    ) -> Result<Self, PresetHandlerError> {
        let name = name.unwrap_or(format!("Preset {}", id));

        display_colors.dedup();

        Ok(Self {
            id,
            name,
            data,
            display_colors,
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

    pub fn stored_attributes(&self, fixture_path: &FixturePath) -> Vec<FixtureChannel3Attribute> {
        match &self.data {
            FixturePresetData::Default { data } => data
                .get(fixture_path)
                .map(|values| values.keys().cloned().collect())
                .unwrap_or_default(),
            FixturePresetData::FeatureEffect { runtime } => {
                runtime.effect().attributes().copied().collect()
            }
            FixturePresetData::KeyframeEffect { runtime } => runtime
                .effect()
                .affected_attributes_for_fixture(fixture_path),
        }
    }

    pub fn apply(
        &self,
        fixture: &Fixture,
        fixture_state: &mut FixtureState,
        new_selection: FixtureSelection,
    ) -> Result<(), PresetHandlerError> {
        match &self.data {
            FixturePresetData::Default { data } => {
                if let Some(fixture_data) = data.get(&fixture.path) {
                    for (attribute, _) in fixture_data.iter() {
                        fixture_state
                            .set_programmer_value(
                                fixture,
                                attribute,
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
                for attribute in runtime
                    .effect()
                    .attributes()
                    .filter(|attr| fixture.has_attribute(attr))
                {
                    fixture_state
                        .set_programmer_value(
                            fixture,
                            attribute,
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
            FixturePresetData::KeyframeEffect { runtime } => {
                for attribute in runtime
                    .effect()
                    .affected_attributes_for_fixture(&fixture.path)
                {
                    fixture_state
                        .set_programmer_value(
                            fixture,
                            &attribute,
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

    pub fn get_target(&self, selected_fixtures: &[FixturePath]) -> FixturePresetTarget {
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
                let affected_fixtures = selected_fixtures
                    .iter()
                    .filter(|f_path| runtime.effect().is_affected(f_path))
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

    pub fn display_colors(&self) -> impl Iterator<Item = FixturePresetDisplayColor> {
        self.display_colors.iter().copied()
    }

    pub fn values(
        &self,
        fixture: &Fixture,
        _preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        state: Option<&FixtureChannelValue2PresetState>,
    ) -> Vec<(FixtureChannel3Attribute, FixtureChannelValue3)> {
        match &self.data {
            FixturePresetData::Default { data } => data
                .get(&fixture.path)
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
                    .and_then(|state| state.selection().offset(&fixture.path))
                    .unwrap_or_default();

                runtime.get_values_with_started(
                    fixture,
                    fixture_offset,
                    timing_handler,
                    state.map(|state| state.started()),
                )
            }
            FixturePresetData::KeyframeEffect { runtime } => {
                let fixture_offset = state
                    .and_then(|state| state.selection().offset(&fixture.path))
                    .unwrap_or_default();

                let channels = runtime
                    .effect()
                    .affected_attributes_for_fixture(&fixture.path);

                channels
                    .iter()
                    .map(|attribute| {
                        (
                            *attribute,
                            runtime
                                .get_attribute_value_with_started(
                                    attribute,
                                    &fixture.path,
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
        fixture: &Fixture,
        attribute: &FixtureChannel3Attribute,
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
                .get(&fixture.path)
                .and_then(|values| values.get(attribute).cloned())
                .map(FixtureChannelValue3::Discrete),
            FixturePresetData::FeatureEffect { runtime } => {
                let fixture_offset = state
                    .and_then(|state| state.selection().offset(&fixture.path))
                    .unwrap_or_default();

                fixture
                    .has_attribute(attribute)
                    .then(|| {
                        runtime
                            .get_channel_value_with_started(
                                attribute,
                                fixture_offset,
                                timing_handler,
                                state.map(|state| state.started()),
                            )
                            .ok()
                    })
                    .flatten()
            }
            FixturePresetData::KeyframeEffect { runtime } => {
                let fixture_offset = state
                    .and_then(|state| state.selection().offset(&fixture.path))
                    .unwrap_or_default();

                runtime
                    .get_attribute_value_with_started(
                        attribute,
                        &fixture.path,
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
        data: HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>>,
        patch: &Patch,
        event_list: &mut DemexEventList,
    ) -> Result<(), PresetHandlerError> {
        match &mut self.data {
            FixturePresetData::FeatureEffect { .. } => {
                Err(PresetHandlerError::PresetCannotRecordNextKeyframe(self.id))
            }
            FixturePresetData::KeyframeEffect { runtime } => {
                runtime.effect_mut().layers_mut()[0].add_keyframe(
                    KeyframeEffectKeyframe::from_data(
                        0.0,
                        data,
                        KeyframeEffectKeyframeCurve::default(),
                        patch,
                    ),
                );

                event_list.push(DemexEvent::PoolItemFlagsUpdated(
                    PoolType::Preset(self.id.feature_group),
                    self.id.preset_id,
                ));

                Ok(())
            }
            FixturePresetData::Default { data: preset_data } => {
                let mut effect = KeyframeEffect::from_data(preset_data.clone(), patch);
                effect.layers_mut()[0].add_keyframe(KeyframeEffectKeyframe::from_data(
                    0.0,
                    data,
                    KeyframeEffectKeyframeCurve::default(),
                    patch,
                ));

                let effect_runtime = KeyframeEffectRuntime::new(
                    effect,
                    EffectSpeed::default(),
                    RuntimePhase::default(),
                );

                self.data = FixturePresetData::KeyframeEffect {
                    runtime: effect_runtime,
                };

                event_list.push(DemexEvent::PoolItemFlagsUpdated(
                    PoolType::Preset(self.id.feature_group),
                    self.id.preset_id,
                ));

                Ok(())
            }
        }
    }

    pub fn update(
        &mut self,
        values_to_update: HashMap<
            FixturePath,
            HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>,
        >,
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

#[repr(u32)]
pub enum FixturePresetFlags {
    KeyframeEffect = 0x1 << 1,
    FeatureEffect = 0x1 << 2,
}

impl From<&FixturePreset> for PoolItem {
    fn from(value: &FixturePreset) -> Self {
        let mut flags = 0;

        match value.data {
            FixturePresetData::FeatureEffect { .. } => {
                flags |= FixturePresetFlags::FeatureEffect as u32
            }
            FixturePresetData::KeyframeEffect { .. } => {
                flags |= FixturePresetFlags::KeyframeEffect as u32
            }
            _ => {}
        };

        PoolItem {
            id: value.id.preset_id,
            name: value.name.clone().into(),
            colors: (!value.display_colors.is_empty()).then(|| {
                value
                    .display_colors
                    .iter()
                    .map(|color| color.rgb_f())
                    .collect()
            }),
            flags: flags,
        }
    }
}
