use std::{collections::HashMap, f32};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3,
        channel_value_discrete::FixtureChannelDiscreteValue,
    },
    fixture::FixturePath,
    keyframe_effect::{
        effect_keyframe::KeyframeEffectKeyframe,
        effect_keyframe_curve::KeyframeEffectKeyframeCurve, effect_layer::KeyframeEffectLayer,
    },
    patch::Patch,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyframeEffect {
    layers: Vec<KeyframeEffectLayer>,
}

impl KeyframeEffect {
    pub fn from_data(
        data: HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>>,
        patch: &Patch,
    ) -> Self {
        let layer = KeyframeEffectLayer::new(vec![KeyframeEffectKeyframe::from_data(
            0.0,
            data,
            KeyframeEffectKeyframeCurve::default(),
            patch,
        )]);

        Self {
            layers: vec![layer],
        }
    }

    pub fn layers_mut(&mut self) -> &mut Vec<KeyframeEffectLayer> {
        &mut self.layers
    }

    pub fn is_affected(&self, fixture_path: &FixturePath) -> bool {
        self.layers
            .iter()
            .any(|layer| layer.is_affected(fixture_path))
    }

    pub fn affected_attributes_for_fixture(
        &self,
        fixture_path: &FixturePath,
    ) -> Vec<FixtureChannel3Attribute> {
        self.layers
            .iter()
            .flat_map(|layer| layer.affected_attributes_for_fixture(fixture_path))
            .dedup()
            .collect()
    }

    pub fn value(
        &self,
        fixture_path: &FixturePath,
        attribute: &FixtureChannel3Attribute,
        started_elapsed: f64,
        phase_offset_deg: f32,
        speed_multiplier: f32,
    ) -> Option<FixtureChannelValue3> {
        let time_adjusted =
            (started_elapsed as f32 * speed_multiplier) - phase_offset_deg.to_radians();

        // convert time_adjusted to a value between 0.0 and 1.0 (from 0.0 to 2π)
        let t = (time_adjusted % (2.0 * f32::consts::PI)) / (2.0 * f32::consts::PI);

        let value = self
            .layers
            .iter()
            .flat_map(|layer| layer.value(fixture_path, attribute, t))
            .next()?;

        Some(FixtureChannelValue3::discrete(value))
    }
}
