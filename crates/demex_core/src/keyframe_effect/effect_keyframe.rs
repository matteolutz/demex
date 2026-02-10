use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value_discrete::FixtureChannelDiscreteValue,
        clamped_value::ClampedValue,
    },
    fixture::FixturePath,
    keyframe_effect::effect_keyframe_curve::KeyframeEffectKeyframeCurve,
    patch::Patch,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyframeEffectKeyframeData {
    Selective(HashMap<FixturePath, HashMap<FixtureChannel3Attribute, ClampedValue>>),
    Global(HashMap<FixtureChannel3Attribute, ClampedValue>),
}

impl From<HashMap<FixturePath, HashMap<FixtureChannel3Attribute, ClampedValue>>>
    for KeyframeEffectKeyframeData
{
    fn from(value: HashMap<FixturePath, HashMap<FixtureChannel3Attribute, ClampedValue>>) -> Self {
        Self::Selective(value)
    }
}

impl Default for KeyframeEffectKeyframeData {
    fn default() -> Self {
        Self::Global(HashMap::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyframeEffectKeyframe {
    pub(crate) starting_point: f32,

    pub(crate) data: KeyframeEffectKeyframeData,

    pub(crate) curve: KeyframeEffectKeyframeCurve,
}

impl KeyframeEffectKeyframe {
    pub fn new(
        starting_point: f32,
        data: impl Into<KeyframeEffectKeyframeData>,
        curve: KeyframeEffectKeyframeCurve,
    ) -> Self {
        Self {
            starting_point,
            data: data.into(),
            curve,
        }
    }

    pub fn from_data(
        starting_point: f32,
        data: HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>>,
        curve: KeyframeEffectKeyframeCurve,
        patch: &Patch,
    ) -> Self {
        Self {
            starting_point,
            data: data
                .into_iter()
                .map(|(f_id, values)| {
                    let fixture = patch.fixture(&f_id).unwrap();

                    (
                        f_id,
                        values
                            .into_iter()
                            .map(|(channel, value)| {
                                (
                                    channel,
                                    value.to_clamped(fixture.channel_function(&channel).unwrap()),
                                )
                            })
                            .collect::<HashMap<_, _>>(),
                    )
                })
                .collect::<HashMap<_, _>>()
                .into(),
            curve,
        }
    }

    pub fn is_global(&self) -> bool {
        matches!(self.data, KeyframeEffectKeyframeData::Global(_))
    }

    pub fn make_global(&mut self) {
        match &self.data {
            KeyframeEffectKeyframeData::Selective(_) => {
                let attributes = self.attributes();
                self.data = KeyframeEffectKeyframeData::Global(
                    attributes
                        .into_iter()
                        .map(|attribute| {
                            let values = self.values_for_attribute(&attribute);
                            let avg: f32 = values.iter().map(|c| c.as_f32()).sum::<f32>()
                                / values.len() as f32;
                            (attribute, avg.into())
                        })
                        .collect(),
                );
            }
            KeyframeEffectKeyframeData::Global(_) => {}
        }
    }

    pub fn starting_point(&self) -> f32 {
        self.starting_point
    }

    pub fn set_starting_point(&mut self, starting_point: f32) {
        self.starting_point = starting_point;
    }

    pub fn curve(&self) -> KeyframeEffectKeyframeCurve {
        self.curve
    }

    pub fn curve_mut(&mut self) -> &mut KeyframeEffectKeyframeCurve {
        &mut self.curve
    }

    pub fn is_affected(&self, fixture_path: &FixturePath) -> bool {
        match &self.data {
            KeyframeEffectKeyframeData::Global(_) => true,
            KeyframeEffectKeyframeData::Selective(values) => values.contains_key(fixture_path),
        }
    }

    fn fixture_data(
        &self,
        fixture_path: &FixturePath,
    ) -> Option<&HashMap<FixtureChannel3Attribute, ClampedValue>> {
        match &self.data {
            KeyframeEffectKeyframeData::Selective(values) => values.get(fixture_path),
            KeyframeEffectKeyframeData::Global(values) => Some(values),
        }
    }

    pub fn affected_attributes_for_fixture(
        &self,
        fixture_path: &FixturePath,
    ) -> Option<Vec<FixtureChannel3Attribute>> {
        Some(self.fixture_data(fixture_path)?.keys().copied().collect())
    }

    pub fn value(
        &self,
        fixture_path: &FixturePath,
        attribute: &FixtureChannel3Attribute,
    ) -> Option<&ClampedValue> {
        self.fixture_data(fixture_path)?.get(attribute)
    }

    pub fn value_at(
        &self,
        fixture_path: &FixturePath,
        attribute: &FixtureChannel3Attribute,
        t: f32,
    ) -> Option<(&ClampedValue, f32)> {
        // t is in the range 0.0..=1.0
        let value = self.value(fixture_path, attribute)?;
        Some((value, self.curve.value(t)))
    }

    pub fn values_for_attribute(&self, attribute: &FixtureChannel3Attribute) -> Vec<ClampedValue> {
        match &self.data {
            KeyframeEffectKeyframeData::Global(values) => values
                .get(attribute)
                .map(|value| vec![*value])
                .unwrap_or_default(),
            KeyframeEffectKeyframeData::Selective(values) => values
                .iter()
                .filter_map(|(_, values)| values.get(attribute).copied())
                .dedup()
                .collect(),
        }
    }

    pub fn attributes(&self) -> HashSet<FixtureChannel3Attribute> {
        match &self.data {
            KeyframeEffectKeyframeData::Global(values) => values.keys().copied().collect(),
            KeyframeEffectKeyframeData::Selective(values) => values
                .iter()
                .flat_map(|(_, values)| values.keys().copied())
                .collect(),
        }
    }
}
