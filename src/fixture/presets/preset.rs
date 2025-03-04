use std::collections::HashMap;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel2::{
            channel_type::FixtureChannelType, channel_value::FixtureChannelValue2,
            feature::feature_group::FeatureGroup,
        },
        error::FixtureError,
        handler::FixtureHandler,
        Fixture,
    },
    parser::nodes::{
        action::{functions::update_function::UpdateMode, ValueOrRange},
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
};

use super::{error::PresetHandlerError, PresetHandler};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default, EguiProbe)]
pub struct FixturePresetId {
    pub feature_group_id: u32,
    pub preset_id: u32,
}

impl ValueOrRange<FixturePresetId> {
    pub fn try_into_id_list(self) -> Result<Vec<FixturePresetId>, PresetHandlerError> {
        match self {
            ValueOrRange::Single(value) => Ok(vec![value]),
            ValueOrRange::Thru(from, to) => {
                if from.feature_group_id != to.feature_group_id {
                    return Err(PresetHandlerError::FeatureGroupMismatch(
                        from.feature_group_id,
                        to.feature_group_id,
                    ));
                }

                Ok((from.preset_id..=to.preset_id)
                    .map(|preset_id| FixturePresetId {
                        feature_group_id: from.feature_group_id,
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
        let ord = self.feature_group_id.cmp(&other.feature_group_id);
        if ord == std::cmp::Ordering::Equal {
            self.preset_id.cmp(&other.preset_id)
        } else {
            ord
        }
    }
}

impl std::fmt::Display for FixturePresetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.feature_group_id, self.preset_id)
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
        let feature_group_id = parts[0].parse().map_err(serde::de::Error::custom)?;
        let preset_id = parts[1].parse().map_err(serde::de::Error::custom)?;
        Ok(Self {
            feature_group_id,
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
pub struct FixturePreset {
    #[egui_probe(skip)]
    id: FixturePresetId,

    name: String,

    #[serde(default)]
    display_color: Option<egui::Color32>,

    #[egui_probe(skip)]
    data: HashMap<u32, HashMap<FixtureChannelType, u8>>,
}

impl FixturePreset {
    pub fn generate_preset_data(
        fixture_handler: &FixtureHandler,
        preset_handler: &PresetHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        feature_group: &FeatureGroup,
    ) -> Result<HashMap<u32, HashMap<FixtureChannelType, u8>>, PresetHandlerError> {
        let mut data: HashMap<u32, HashMap<FixtureChannelType, u8>> = HashMap::new();

        for fixture_id in fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?
            .fixtures()
        {
            let fixture = fixture_handler.fixture_immut(*fixture_id);
            if let Some(fixture) = fixture {
                let mut new_values = HashMap::new();

                for feature_type in feature_group.feature_types() {
                    if !fixture.feature_types().contains(feature_type) {
                        continue;
                    }

                    if fixture
                        .feature_is_home_programmer(*feature_type)
                        .map_err(PresetHandlerError::FixtureError)?
                    {
                        continue;
                    }

                    for channel_type in fixture
                        .feature_get_channel_types(*feature_type)
                        .map_err(PresetHandlerError::FixtureError)?
                    {
                        let discrete_value = fixture
                            .channel_value_programmer(channel_type)
                            .and_then(|value| {
                                value
                                    .to_discrete_value(*fixture_id, channel_type, preset_handler)
                                    .map_err(FixtureError::FixtureChannelError2)
                            })
                            .map_err(PresetHandlerError::FixtureError)?;

                        new_values.insert(channel_type, discrete_value);
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
        feature_group: &FeatureGroup,
        data: HashMap<u32, HashMap<FixtureChannelType, u8>>,
    ) -> Result<Self, PresetHandlerError> {
        let name = name.unwrap_or(format!("{} Preset {}", feature_group.name(), id));

        Ok(Self {
            id,
            name,
            data,
            display_color: None,
        })
    }

    pub fn apply(&self, fixture: &mut Fixture) -> Result<(), PresetHandlerError> {
        if let Some(fixture_data) = self.data.get(&fixture.id()) {
            for (preset_chanel_type, _) in fixture_data.iter() {
                fixture
                    .set_channel_value(*preset_chanel_type, FixtureChannelValue2::Preset(self.id))
                    .map_err(PresetHandlerError::FixtureError)?;
            }
        }

        Ok(())
    }

    pub fn get_target(&self, selected_fixtures: &[u32]) -> FixturePresetTarget {
        let mutual = self
            .data
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

    pub fn value(&self, fixture_id: u32, channel_type: FixtureChannelType) -> Option<u8> {
        self.data
            .get(&fixture_id)
            .and_then(|values| values.get(&channel_type).copied())
    }

    pub fn update(
        &mut self,
        values_to_update: HashMap<u32, HashMap<FixtureChannelType, u8>>,
        update_mode: UpdateMode,
    ) -> Result<usize, PresetHandlerError> {
        let mut updated = 0;

        for (fixture_id, new_fixture_values) in values_to_update {
            // if we already have a value for this fixture and we are not in override mode, skip
            if self.data.contains_key(&fixture_id) && update_mode != UpdateMode::Override {
                continue;
            }

            // Insert or update
            self.data.insert(fixture_id, new_fixture_values);

            updated += 1;
        }

        Ok(updated)
    }
}
