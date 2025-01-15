use std::fmt;

use itertools::Itertools;

use super::{
    channel::value::{FixtureChannelDiscreteValue, FixtureChannelValue},
    error::FixtureError,
    presets::PresetHandler,
    sequence::FadeFixtureChannelValue,
    Fixture,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureChannelValuePriority {
    LTP,
}

impl FixtureChannelValuePriority {
    pub fn priority_value(&self) -> u8 {
        match self {
            Self::LTP => 0,
        }
    }
}

impl PartialOrd for FixtureChannelValuePriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority_value().cmp(&other.priority_value()))
    }
}

impl Ord for FixtureChannelValuePriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority_value().cmp(&other.priority_value())
    }
}

pub trait FixtureChannelValueSourceTrait {
    fn get_channel_value(
        &self,
        fixture: &Fixture,
        channel_id: u16,
        preset_handler: &PresetHandler,
    ) -> Result<FixtureChannelValue, FixtureError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureChannelValueSource {
    Programmer,
    SequenceRuntime { runtime_id: u32 },
}

impl FixtureChannelValueSource {
    pub fn priority(&self) -> FixtureChannelValuePriority {
        match self {
            Self::Programmer => FixtureChannelValuePriority::LTP,
            // TODO
            Self::SequenceRuntime { runtime_id: _ } => FixtureChannelValuePriority::LTP,
        }
    }
}

impl FixtureChannelValueSourceTrait for Vec<FixtureChannelValueSource> {
    fn get_channel_value(
        &self,
        fixture: &Fixture,
        channel_id: u16,
        preset_handler: &PresetHandler,
    ) -> Result<FixtureChannelValue, FixtureError> {
        let values = self
            .iter()
            .sorted_by(|a, b| a.priority().cmp(&b.priority()))
            .flat_map(|source| match source {
                FixtureChannelValueSource::Programmer => fixture
                    .channel_value_programmer(channel_id)
                    .map(|v| FadeFixtureChannelValue::new(v, 1.0)),
                FixtureChannelValueSource::SequenceRuntime { runtime_id } => {
                    let runtime = preset_handler.sequence_runtime(*runtime_id);

                    if let Some(runtime) = runtime {
                        runtime
                            .channel_value(fixture.id(), channel_id)
                            .ok_or(FixtureError::ChannelValueNotFound(channel_id))
                    } else {
                        Err(FixtureError::ChannelValueNotFound(channel_id))
                    }
                }
            })
            .collect::<Vec<_>>();

        if values.is_empty() {
            return Err(FixtureError::ChannelValueNotFound(channel_id));
        }

        let mut value = FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::AnyHome);
        for v in values {
            if v.alpha() == 0.0 {
                continue;
            }

            if v.alpha() == 1.0 {
                value = v.value().clone();
                continue;
            }

            value = FixtureChannelValue::Mix {
                a: Box::new(value),
                b: Box::new(v.value().clone()),
                mix: v.alpha(),
            };
        }

        Ok(value)
    }
}

impl fmt::Display for FixtureChannelValueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Programmer => write!(f, "PRG"),
            Self::SequenceRuntime { runtime_id } => write!(f, "SR({})", runtime_id),
        }
    }
}
