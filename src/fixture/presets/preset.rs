use std::{collections::HashMap, str::FromStr};

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel3::{
            channel_value::{FixtureChannelValue2PresetState, FixtureChannelValue3},
            feature::{
                feature_group::FixtureChannel3FeatureGroup,
                feature_type::FixtureChannel3FeatureType,
            },
        },
        effect::feature::runtime::FeatureEffectRuntime,
        gdtf::GdtfFixture,
        handler::{FixtureHandler, FixtureTypeList},
        selection::FixtureSelection,
        timing::TimingHandler,
        value_source::FixtureChannelValuePriority,
    },
    parser::nodes::{
        action::{functions::update_function::UpdateMode, ValueOrRange},
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
};

use super::{error::PresetHandlerError, PresetHandler};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default, EguiProbe)]
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

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub enum FixturePresetData {
    Default {
        #[egui_probe(skip)]
        data: HashMap<u32, HashMap<String, FixtureChannelValue3>>,
    },
    FeatureEffect {
        runtime: FeatureEffectRuntime,
    },
}

impl Default for FixturePresetData {
    fn default() -> Self {
        Self::Default {
            data: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub struct FixturePreset {
    #[egui_probe(skip)]
    id: FixturePresetId,

    name: String,

    #[serde(default)]
    display_color: Option<egui::Color32>,

    #[serde(default)]
    fade_up: f32,

    data: FixturePresetData,
}

impl FixturePreset {
    pub fn generate_preset_data(
        fixture_types: &FixtureTypeList,
        fixture_handler: &FixtureHandler,
        preset_handler: &mut PresetHandler,
        timing_handler: &TimingHandler,
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
            let fixture = fixture_handler.fixture_immut(*fixture_id);

            if let Some(fixture) = fixture {
                let mut new_values = HashMap::new();

                let (fixture_type, dmx_mode) =
                    fixture.fixture_type_and_dmx_mode(fixture_types).unwrap();

                for dmx_channel in &dmx_mode.dmx_channels {
                    // let channel = dmx_channel.logical_channels[0];

                    // check, if the channel attribute belongs into the correct feature group
                    if dmx_channel.logical_channels[0]
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

                    let value = fixture.get_programmer_value(dmx_channel.name().as_ref());

                    if let Ok(value) = value {
                        if value.is_home() {
                            continue;
                        }

                        new_values.insert(
                            dmx_channel.name().as_ref().to_owned(),
                            value.clone().to_discrete(
                                fixture,
                                fixture_types,
                                dmx_channel.name().as_ref(),
                                preset_handler,
                                timing_handler,
                            ),
                        );
                    }
                }

                // if we have values for this fixture, insert them
                if !new_values.is_empty() {
                    data.insert(*fixture_id, new_values);
                }
            }
        }

        Ok(data)
    }

    pub fn new(
        id: FixturePresetId,
        name: Option<String>,
        feature_group: FixtureChannel3FeatureGroup,
        data: FixturePresetData,
    ) -> Result<Self, PresetHandlerError> {
        let name = name.unwrap_or(format!("{} Preset {}", feature_group.name(), id));

        Ok(Self {
            id,
            name,
            data,
            display_color: None,
            fade_up: 0.0,
        })
    }

    pub fn data(&self) -> &FixturePresetData {
        &self.data
    }

    pub fn apply(
        &mut self,
        fixture_types: &FixtureTypeList,
        fixture: &mut GdtfFixture,
        new_selection: FixtureSelection,
    ) -> Result<(), PresetHandlerError> {
        match &mut self.data {
            FixturePresetData::Default { data } => {
                if let Some(fixture_data) = data.get(&fixture.id()) {
                    for (preset_chanel_type, _) in fixture_data.iter() {
                        fixture
                            .set_programmer_value(
                                fixture_types,
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
                for attribute in runtime.effect().get_attributes() {
                    // if the fixture doesn't have this feature type, skip
                    if let Ok(channels) = fixture.channels_for_attribute(fixture_types, attribute) {
                        for (dmx_channel, _, _) in channels {
                            fixture
                                .set_programmer_value(
                                    fixture_types,
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
        }
    }

    pub fn id(&self) -> FixturePresetId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn display_color(&self) -> Option<egui::Color32> {
        self.display_color
    }

    pub fn value(
        &self,
        fixture: &GdtfFixture,
        fixture_types: &FixtureTypeList,
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
                .and_then(|values| values.get(channel_name).cloned()),
            FixturePresetData::FeatureEffect { runtime } => {
                let fixture_offset = state
                    .and_then(|state| state.selection().offset(fixture.id()))
                    .unwrap_or_default();

                let fade_val = runtime.get_channel_value_with_started(
                    channel_name,
                    fixture,
                    fixture_types,
                    fixture_offset,
                    FixtureChannelValuePriority::default(),
                    timing_handler,
                    state.map(|state| state.started()),
                );

                fade_val.ok().map(|value| value.value().clone())
            }
        };

        // val.map(|val| ((val as f32 / 255.0) * fade * 255.0) as u8)
        val
    }

    pub fn update(
        &mut self,
        values_to_update: HashMap<u32, HashMap<String, FixtureChannelValue3>>,
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
            FixturePresetData::FeatureEffect { .. } => todo!(),
        }
    }
}
