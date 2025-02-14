use std::fmt;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    error::FixtureError, presets::PresetHandler, sequence::FadeFixtureChannelValue,
    updatables::UpdatableHandler, Fixture,
};

use super::channel2::{channel_type::FixtureChannelType, channel_value::FixtureChannelValue2};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, EguiProbe, Default)]
pub enum FixtureChannelValuePriority {
    #[default]
    Ltp,
    SuperLtp,
    Htp,
}

impl FixtureChannelValuePriority {
    pub fn priority_value(&self) -> u8 {
        match self {
            Self::SuperLtp => 1,
            Self::Ltp => 0,
            Self::Htp => 0,
        }
    }

    pub fn programmer() -> Self {
        Self::Ltp
    }

    pub fn is_htp(&self) -> bool {
        matches!(self, Self::Htp)
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
        channel_type: FixtureChannelType,
        updatable_handler: &UpdatableHandler,
        preset_handler: &PresetHandler,
    ) -> Result<FixtureChannelValue2, FixtureError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureChannelValueSource {
    Programmer,
    Executor { executor_id: u32 },
    Fader { fader_id: u32 },
}

impl FixtureChannelValueSourceTrait for Vec<FixtureChannelValueSource> {
    fn get_channel_value(
        &self,
        fixture: &Fixture,
        channel_type: FixtureChannelType,
        updatable_handler: &UpdatableHandler,
        preset_handler: &PresetHandler,
    ) -> Result<FixtureChannelValue2, FixtureError> {
        let mut values = self
            .iter()
            .flat_map(|source| match source {
                FixtureChannelValueSource::Programmer => {
                    fixture.channel_value_programmer(channel_type).map(|v| {
                        FadeFixtureChannelValue::new(
                            v.clone(),
                            1.0,
                            FixtureChannelValuePriority::programmer(),
                        )
                    })
                }
                FixtureChannelValueSource::Executor {
                    executor_id: runtime_id,
                } => {
                    let runtime = updatable_handler.executor(*runtime_id);

                    if let Some(runtime) = runtime {
                        runtime
                            .channel_value(fixture.id(), channel_type, preset_handler)
                            .ok_or(FixtureError::ChannelValueNotFound(channel_type))
                    } else {
                        Err(FixtureError::ChannelValueNotFound(channel_type))
                    }
                }
                FixtureChannelValueSource::Fader { fader_id: id } => {
                    let fader = updatable_handler.fader(*id);

                    if let Ok(fader) = fader {
                        fader.get_channel_value(fixture, channel_type, preset_handler)
                    } else {
                        Err(FixtureError::ChannelValueNotFound(channel_type))
                    }
                }
            })
            .collect::<Vec<_>>();

        if values.is_empty() {
            return Err(FixtureError::ChannelValueNotFound(channel_type));
        }

        values.sort_by_key(|v| v.priority());

        let mut value = FixtureChannelValue2::Home;

        for v in values {
            if v.value().is_home() {
                continue;
            }

            if !v.priority().is_htp() {
                if v.alpha() == 0.0 {
                    value = FixtureChannelValue2::Home;
                } else if v.alpha() == 1.0 {
                    value = v.value().clone()
                } else {
                    value = FixtureChannelValue2::Mix {
                        a: Box::new(FixtureChannelValue2::Home),
                        b: Box::new(v.value().clone()),
                        mix: v.alpha(),
                    };
                }

                continue;
            }

            if v.alpha() == 0.0 {
                continue;
            }

            if v.alpha() == 1.0 {
                value = v.value().clone();
                continue;
            }

            value = FixtureChannelValue2::Mix {
                a: Box::new(value),
                b: Box::new(v.value().clone()),
                mix: v.alpha(),
            };
        }

        Ok(value)
    }
}

impl FixtureChannelValueSource {
    pub fn to_short_string(&self) -> String {
        match self {
            Self::Programmer => "P".to_string(),
            Self::Executor { executor_id } => executor_id.to_string(),
            Self::Fader { fader_id } => fader_id.to_string(),
        }
    }
}

impl fmt::Display for FixtureChannelValueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Programmer => write!(f, "Prg"),
            Self::Executor {
                executor_id: runtime_id,
            } => write!(f, "Exe({})", runtime_id),
            Self::Fader { fader_id } => write!(f, "Fdr({})", fader_id),
        }
    }
}
