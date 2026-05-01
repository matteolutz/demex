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
        effect_preset::KeyframeEffectPreset,
    },
    patch::Patch,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyframeEffect {
    layers: Vec<KeyframeEffectLayer>,

    preset: Option<KeyframeEffectPreset>,
}

impl KeyframeEffect {
    pub fn new() -> Self {
        Self {
            layers: vec![],
            preset: None,
        }
    }

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
            preset: None,
        }
    }

    pub fn from_preset(preset: KeyframeEffectPreset) -> Self {
        Self {
            layers: preset.build_layers(),
            preset: Some(preset),
        }
    }

    pub fn preset(&self) -> Option<&KeyframeEffectPreset> {
        self.preset.as_ref()
    }

    pub fn apply_preset(&mut self, preset: KeyframeEffectPreset) {
        self.layers = preset.build_layers();
        self.preset = Some(preset);
    }

    pub fn is_global(&self) -> bool {
        self.layers.iter().all(|layer| layer.is_global())
    }

    pub fn make_global(&mut self) {
        self.layers.iter_mut().for_each(|layer| layer.make_global());
    }

    pub fn layers(&self) -> &[KeyframeEffectLayer] {
        &self.layers
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
        phase: f32,
    ) -> Option<FixtureChannelValue3> {
        /*let time_adjusted =
        (started_elapsed as f32 * speed_multiplier) - phase_offset_deg.to_radians();*/

        let value = self
            .layers
            .iter()
            .flat_map(|layer| layer.value(fixture_path, attribute, phase))
            .next()?;

        Some(FixtureChannelValue3::discrete(value))
    }
}
