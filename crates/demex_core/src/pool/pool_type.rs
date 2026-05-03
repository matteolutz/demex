use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    channel3::feature::feature_group::FixtureChannel3FeatureGroup,
    command::parser::nodes::object::{HomeableObject, Object, ObjectType},
    presets::preset::FixturePresetId,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolType {
    Executor,
    Preset(FixtureChannel3FeatureGroup),
    Sequence,
    SequenceCue(u32),
    Group,
    Macro,
}

impl Default for PoolType {
    fn default() -> Self {
        PoolType::Preset(FixtureChannel3FeatureGroup::default())
    }
}

impl PoolType {
    pub fn all() -> Vec<PoolType> {
        [Self::Executor, Self::Sequence, Self::Group, Self::Macro]
            .into_iter()
            .chain(FixtureChannel3FeatureGroup::iter().map(|fg| Self::Preset(fg)))
            .collect()
    }

    pub fn get_object(&self, id: u32) -> Object {
        match self {
            Self::Macro => Object::Macro(id),
            Self::Executor => HomeableObject::Executor(id).into(),
            Self::Group => HomeableObject::Group(id).into(),
            &Self::Preset(feature_group) => Object::Preset(FixturePresetId::new(feature_group, id)),
            Self::Sequence => Object::Sequence(id),
            &Self::SequenceCue(seq_id) => Object::SequenceCue(seq_id, id.into()),
        }
    }
}

impl From<PoolType> for ObjectType {
    fn from(value: PoolType) -> Self {
        match value {
            PoolType::Executor => ObjectType::Executor,
            PoolType::Preset(_) => ObjectType::Preset,
            PoolType::Sequence => ObjectType::Sequence,
            PoolType::SequenceCue(_) => ObjectType::SequenceCue,
            PoolType::Group => ObjectType::Group,
            PoolType::Macro => ObjectType::Macro,
        }
    }
}
