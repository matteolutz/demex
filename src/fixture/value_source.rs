use std::fmt;

use serde::{Deserialize, Serialize};

use crate::fixture::{
    error::FixtureError, presets::PresetHandler, sequence::FadeFixtureChannelValue,
    updatables::UpdatableHandler,
};

use super::{
    channel3::channel_value::FixtureChannelValue3, gdtf::GdtfFixture, handler::FixtureTypeList,
    timing::TimingHandler, updatables::StompSource,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
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
        fixture_types: &FixtureTypeList,
        fixture: &GdtfFixture,
        channel: &gdtf::dmx_mode::DmxChannel,
        updatable_handler: &UpdatableHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureChannelValue3, FixtureError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixtureChannelValueSource {
    Programmer,
    Executor { executor_id: u32 },
}

impl FixtureChannelValueSource {
    fn is_stomped_by(
        &self,
        updatable_handler: &UpdatableHandler,
        stomp_source: &Option<StompSource>,
    ) -> Result<bool, FixtureError> {
        match self {
            Self::Programmer => {
                // TODO: should the programmer be stomp protected?
                // Maybe we should add a setting for this
                let programmer_stomp_protected = true;

                Ok(!programmer_stomp_protected)
            }
            Self::Executor { executor_id } => {
                let executor = updatable_handler
                    .executor(*executor_id)
                    .map_err(|err| FixtureError::UpdatableHandlerError(err.into()))?;

                Ok(!executor.stomp_protected()
                    && stomp_source.as_ref().is_some_and(
                        |s| !matches!(s, StompSource::Executor(id) if id == executor_id),
                    ))
            }
        }
    }
}

impl FixtureChannelValueSourceTrait for Vec<FixtureChannelValueSource> {
    fn get_channel_value(
        &self,
        fixture_types: &FixtureTypeList,
        fixture: &GdtfFixture,
        channel: &gdtf::dmx_mode::DmxChannel,
        updatable_handler: &UpdatableHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureChannelValue3, FixtureError> {
        let last_stomp_source = updatable_handler.last_stomp_source();

        let mut values = self
            .iter()
            .flat_map(|source| {
                if source.is_stomped_by(updatable_handler, &last_stomp_source)? {
                    Ok(FadeFixtureChannelValue::home_ltp())
                } else {
                    match source {
                        FixtureChannelValueSource::Programmer => fixture
                            .get_programmer_value(channel.name().as_ref())
                            .map(|v| {
                                FadeFixtureChannelValue::new(
                                    v.clone(),
                                    1.0,
                                    FixtureChannelValuePriority::programmer(),
                                )
                            }),
                        FixtureChannelValueSource::Executor { executor_id } => {
                            let executor = updatable_handler.executor(*executor_id);

                            if let Ok(executor) = executor {
                                executor.channel_value(
                                    fixture_types,
                                    fixture,
                                    channel,
                                    preset_handler,
                                    timing_handler,
                                )
                            } else {
                                Err(FixtureError::GdtfChannelValueNotFound(
                                    channel.name().as_ref().to_owned(),
                                ))
                            }
                        }
                    }
                }
            })
            .map(|val| val.flatten_value())
            .collect::<Vec<_>>();

        if values.is_empty() {
            return Err(FixtureError::GdtfChannelValueNotFound(
                channel.name().as_ref().to_owned(),
            ));
        }

        values.sort_by_key(|v| v.priority());

        let mut value = FixtureChannelValue3::Home;

        for v in values {
            if v.value().is_home() {
                continue;
            }

            if !v.priority().is_htp() {
                if v.alpha() == 0.0 {
                    value = FixtureChannelValue3::Home;
                } else if v.alpha() == 1.0 {
                    value = v.value().clone()
                } else {
                    value = FixtureChannelValue3::Mix {
                        a: Box::new(FixtureChannelValue3::Home),
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

            value = FixtureChannelValue3::Mix {
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
        }
    }
}
