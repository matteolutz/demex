use std::collections::{hash_map::Keys, HashMap};

use channel2::{
    channel_modifier::{FixtureChannelModifier, FixtureChannelModifierTrait},
    channel_type::FixtureChannelType,
    channel_value::FixtureChannelValue2,
    color::color_gel::ColorGelTrait,
    feature::{
        feature_config::FixtureFeatureConfig, feature_state::FixtureFeatureDisplayState,
        feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue, IntoFeatureType,
    },
};
use patch::{FixturePatchType, FixturePatchTypeMode, FixtureTypeAndMode};
use presets::PresetHandler;
use serde::{Deserialize, Serialize};
use timing::TimingHandler;
use updatables::UpdatableHandler;
use value_source::{FixtureChannelValueSource, FixtureChannelValueSourceTrait};

use self::error::FixtureError;

pub mod channel2;
pub mod effect;
pub mod error;
pub mod gdtf;
pub mod handler;
pub mod layout;
pub mod patch;
pub mod presets;
pub mod selection;
pub mod sequence;
pub mod timing;
pub mod updatables;
pub mod value_source;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableFixturePatch {
    pub id: u32,
    pub name: String,

    // TODO: refactor this, to use FixtureTypeAndMode
    pub fixture_type: String,
    pub fixture_mode: u32,

    #[serde(default)]
    pub channel_modifiers: Vec<FixtureChannelModifier>,

    pub universe: u16,
    pub start_address: u16,
}

impl From<Fixture> for SerializableFixturePatch {
    fn from(value: Fixture) -> Self {
        Self {
            id: value.id,
            name: value.name,
            fixture_type: value.fixture_type.name,
            fixture_mode: value.fixture_type.mode,
            universe: value.universe,
            start_address: value.start_address,
            channel_modifiers: value.channel_modifiers,
        }
    }
}

impl SerializableFixturePatch {
    pub fn try_into_fixture(
        self,
        fixture_types: &HashMap<String, FixturePatchType>,
    ) -> Result<Fixture, FixtureError> {
        let patch = fixture_types
            .get(&self.fixture_type)
            .ok_or(FixtureError::FixtureTypeNotFound(self.fixture_type.clone()))?
            .modes
            .get(&self.fixture_mode)
            .ok_or(FixtureError::FixtureTypeModeNotFound(
                self.fixture_type.clone(),
                self.fixture_mode,
            ))?;

        Fixture::new(
            self.id,
            self.name,
            patch.clone(),
            FixtureTypeAndMode {
                name: self.fixture_type,
                mode: self.fixture_mode,
            },
            self.channel_modifiers,
            self.universe,
            self.start_address,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Fixture {
    id: u32,
    name: String,

    fixture_type: FixtureTypeAndMode,

    channels: Vec<(FixtureChannelType, FixtureChannelValue2)>,
    feature_configs: Vec<FixtureFeatureConfig>,

    channel_modifiers: Vec<FixtureChannelModifier>,

    toggle_flags: Vec<HashMap<String, u8>>,

    universe: u16,
    start_address: u16,
    sources: Vec<FixtureChannelValueSource>,
}

impl Fixture {
    pub fn new(
        id: u32,
        name: String,
        patch: FixturePatchTypeMode,
        fixture_type: FixtureTypeAndMode,
        channel_modifiers: Vec<FixtureChannelModifier>,
        universe: u16,
        start_address: u16,
    ) -> Result<Self, FixtureError> {
        // validate, that the patch is not empty
        if patch.channel_types.is_empty() {
            return Err(FixtureError::EmptyPatch);
        }

        /*if patch
            .channel_types
            .iter()
            .filter(|channel_type| **channel_type != FixtureChannelType::Unused)
            .count()
            != patch
                .channel_types
                .iter()
                .filter(|channel_type| **channel_type != FixtureChannelType::Unused)
                .unique()
                .count()
        {
            return Err(FixtureError::DuplicateChannelType);
        }*/

        Ok(Self {
            id,
            name,
            channels: patch
                .channel_types
                .into_iter()
                .map(|v| (v, FixtureChannelValue2::default()))
                .collect::<Vec<_>>(),
            toggle_flags: patch.toggle_flags,
            feature_configs: patch.feature_configs,
            fixture_type,
            universe,
            start_address,
            channel_modifiers,
            sources: vec![FixtureChannelValueSource::Programmer],
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fixture_type(&self) -> &FixtureTypeAndMode {
        &self.fixture_type
    }

    pub fn channels(&self) -> &[(FixtureChannelType, FixtureChannelValue2)] {
        &self.channels
    }

    pub fn channel_types(&self) -> Vec<&FixtureChannelType> {
        self.channels
            .iter()
            .map(|(channel_type, _)| channel_type)
            .collect::<Vec<_>>()
    }

    pub fn feature_configs(&self) -> &[FixtureFeatureConfig] {
        &self.feature_configs
    }

    pub fn feature_types(&self) -> Vec<FixtureFeatureType> {
        self.feature_configs
            .iter()
            .map(|config| config.feature_type())
            .collect::<Vec<_>>()
    }

    pub fn feature_config_by_type(
        &self,
        feature_type: FixtureFeatureType,
    ) -> Option<&FixtureFeatureConfig> {
        self.feature_configs
            .iter()
            .find(|config| config.feature_type() == feature_type)
    }

    pub fn universe(&self) -> u16 {
        self.universe
    }

    pub fn start_address(&self) -> u16 {
        self.start_address
    }

    pub fn toggle_flag_names(&self, toggle_flag_idx: usize) -> Option<Keys<'_, String, u8>> {
        if toggle_flag_idx >= self.toggle_flags.len() {
            None
        } else {
            Some(self.toggle_flags[toggle_flag_idx].keys())
        }
    }

    pub fn generate_data_packet(
        &self,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
        _grand_master: f32,
    ) -> Result<Vec<u8>, FixtureError> {
        let mut data = Vec::with_capacity(self.channels.len());

        // TODO: get data from other sources
        // TODO: optimize this
        for (channel_type, _) in &self.channels {
            let mut discrete_value = self
                .sources
                .get_channel_value(
                    self,
                    *channel_type,
                    updatable_handler,
                    preset_handler,
                    timing_handler,
                )?
                .to_discrete_value(self, *channel_type, preset_handler, timing_handler)
                .map_err(FixtureError::FixtureChannelError2)?;

            // apply modifiers
            discrete_value = self.channel_modifiers.apply(*channel_type, discrete_value);

            data.push(discrete_value);
        }

        Ok(data)
    }

    pub fn sources(&self) -> &[FixtureChannelValueSource] {
        &self.sources
    }
}

impl Fixture {
    pub fn is_home(&self) -> bool {
        // TODO: change this
        self.channels.iter().all(|(_, val)| val.is_home())
    }

    pub fn push_value_source(&mut self, value_source: FixtureChannelValueSource) {
        self.sources.retain(|source| !source.eq(&value_source));
        self.sources.push(value_source);
    }

    pub fn remove_value_source(&mut self, value_source: FixtureChannelValueSource) {
        self.sources.retain(|source| source != &value_source);
    }

    pub fn display_color(
        &self,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<[f32; 3], FixtureError> {
        if let Ok(FixtureFeatureValue::ColorWheel { wheel_value }) = self.feature_value(
            FixtureFeatureType::ColorWheel,
            preset_handler,
            updatable_handler,
            timing_handler,
        ) {
            if let Some(FixtureFeatureConfig::ColorWheel { wheel_config }) =
                self.feature_config_by_type(FixtureFeatureType::ColorWheel)
            {
                if let Some(rgb_color) = wheel_config
                    .try_get_as_macro(&wheel_value)
                    .map(|macro_val| macro_val.get_rgb())
                {
                    return Ok(rgb_color);
                }
            }
        }

        if let Ok(FixtureFeatureValue::ColorRGB { r, g, b }) = self.feature_value(
            FixtureFeatureType::ColorRGB,
            preset_handler,
            updatable_handler,
            timing_handler,
        ) {
            return Ok([r, g, b]);
        }

        Err(FixtureError::NoDisplayColor(self.id))
    }

    pub fn feature_display_state(
        &self,
        feature_type: FixtureFeatureType,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureFeatureDisplayState, FixtureError> {
        feature_type
            .get_display_state(
                self,
                &self.feature_configs,
                &(|channel_type| {
                    self.sources
                        .get_channel_value(
                            self,
                            channel_type,
                            updatable_handler,
                            preset_handler,
                            timing_handler,
                        )
                        .ok()
                }),
                preset_handler,
                timing_handler,
            )
            .map_err(FixtureError::FixtureChannelError2)
    }

    pub fn feature_group_display_state(
        &self,
        feature_group_id: u32,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<Vec<FixtureFeatureDisplayState>, FixtureError> {
        preset_handler
            .get_feature_group(feature_group_id)
            .map_err(|err| FixtureError::PresetHandlerError(Box::new(err)))?
            .get_display_state(
                self,
                &self.feature_configs,
                &(|channel_type| {
                    self.sources
                        .get_channel_value(
                            self,
                            channel_type,
                            updatable_handler,
                            preset_handler,
                            timing_handler,
                        )
                        .ok()
                }),
                preset_handler,
                timing_handler,
            )
            .map_err(FixtureError::FixtureChannelError2)
    }

    pub fn channel_value_programmer(
        &self,
        find_channel_type: FixtureChannelType,
    ) -> Result<&FixtureChannelValue2, FixtureError> {
        self.channels
            .iter()
            .find(|(channel_type, _)| *channel_type == find_channel_type)
            .map(|(_, channel_value)| channel_value)
            .ok_or(FixtureError::ChannelNotFound(find_channel_type))
    }

    pub fn channel_value(
        &self,
        channel_type: FixtureChannelType,
        updatable_handler: &UpdatableHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureChannelValue2, FixtureError> {
        self.sources.get_channel_value(
            self,
            channel_type,
            updatable_handler,
            preset_handler,
            timing_handler,
        )
    }

    pub fn set_channel_value(
        &mut self,
        find_channel_type: FixtureChannelType,
        value: FixtureChannelValue2,
    ) -> Result<(), FixtureError> {
        // make the programmer the first element in the sources vector
        self.push_value_source(FixtureChannelValueSource::Programmer);

        let channel_value = self
            .channels
            .iter_mut()
            .find(|(channel_type, _)| *channel_type == find_channel_type)
            .map(|(_, channel_value)| channel_value)
            .ok_or(FixtureError::ChannelNotFound(find_channel_type))?;
        *channel_value = value;

        Ok(())
    }

    pub fn feature_value_programmer(
        &self,
        feature_type: FixtureFeatureType,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureFeatureValue, FixtureError> {
        feature_type
            .get_value(
                &self.feature_configs,
                &(|channel_type| {
                    self.channels
                        .iter()
                        .find(|(find_channel_type, _)| *find_channel_type == channel_type)
                        .map(|(_, value)| value)
                        .cloned()
                }),
                self,
                preset_handler,
                timing_handler,
            )
            .map_err(FixtureError::FixtureChannelError2)
    }

    pub fn feature_value(
        &self,
        feature_type: FixtureFeatureType,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureFeatureValue, FixtureError> {
        feature_type
            .get_value(
                &self.feature_configs,
                &(|channel_type| {
                    self.sources
                        .get_channel_value(
                            self,
                            channel_type,
                            updatable_handler,
                            preset_handler,
                            timing_handler,
                        )
                        .ok()
                }),
                self,
                preset_handler,
                timing_handler,
            )
            .map_err(FixtureError::FixtureChannelError2)
    }

    pub fn set_feature_value(
        &mut self,
        feature_value: FixtureFeatureValue,
    ) -> Result<(), FixtureError> {
        // make the programmer the first element in the sources vector
        self.push_value_source(FixtureChannelValueSource::Programmer);

        feature_value
            .write_back(&self.feature_configs, &mut self.channels)
            .map_err(FixtureError::FixtureChannelError2)
    }

    pub fn home_feature(&mut self, feature_type: FixtureFeatureType) -> Result<(), FixtureError> {
        // make the programmer the first element in the sources vector
        self.push_value_source(FixtureChannelValueSource::Programmer);

        feature_type
            .home(&self.feature_configs, &mut self.channels)
            .map_err(FixtureError::FixtureChannelError2)
    }

    pub fn feature_is_home_programmer(
        &self,
        feature_type: FixtureFeatureType,
    ) -> Result<bool, FixtureError> {
        feature_type
            .is_home(
                &self.feature_configs,
                &(|channel_type| {
                    self.channels
                        .iter()
                        .find(|(find_channel_type, _)| *find_channel_type == channel_type)
                        .map(|(_, value)| value)
                        .cloned()
                }),
            )
            .map_err(FixtureError::FixtureChannelError2)
    }

    pub fn feature_get_channel_types(
        &self,
        feature_type: FixtureFeatureType,
    ) -> Result<Vec<FixtureChannelType>, FixtureError> {
        feature_type
            .get_channel_types(&self.feature_configs)
            .map_err(FixtureError::FixtureChannelError2)
    }
}

impl Fixture {
    pub fn home(&mut self, clear_sources: bool) -> Result<(), FixtureError> {
        if clear_sources {
            // remove every source except the programmer
            self.sources.clear();
            self.sources.push(FixtureChannelValueSource::Programmer);
        }

        for (_, channel_value) in self.channels.iter_mut() {
            *channel_value = FixtureChannelValue2::default();
        }

        Ok(())
    }

    pub fn set_toggle_flag(&mut self, _flag_name: &str) -> Result<(), FixtureError> {
        todo!()
    }

    pub fn unset_toggle_flags(&mut self) -> Result<(), FixtureError> {
        todo!();
    }
}

impl Fixture {
    pub fn to_string(
        &self,
        _preset_handler: &PresetHandler,
        _updatable_handler: &UpdatableHandler,
    ) -> String {
        format!(
            "{}\n{} (U{}.{})",
            self.name, self.id, self.universe, self.start_address
        )
    }
}
