use std::collections::HashSet;

use fader_function::DemexExecutorFaderFunction;
use serde::{Deserialize, Serialize};

pub mod fader_function;

use crate::{
    channel3::attribute::FixtureChannel3Attribute,
    event::{DemexEvent, list::DemexEventList},
    fixture::{FixturePath, error::FixtureError},
    implement_set_property,
    patch::Patch,
    pool::{PoolItem, PoolItemName, PoolType},
    presets::PresetHandler,
    sequence::{FadeFixtureChannelValue, runtime::SequenceRuntime},
    state::fixture_state_handler::FixtureStateHandler,
    timing::TimingHandler,
    value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DemexExecutor {
    id: u32,

    priority: FixtureChannelValuePriority,

    #[serde(default)]
    stomp_protected: bool,

    #[serde(default, skip_serializing)]
    value: f32,

    runtime: SequenceRuntime,
    fader_function: DemexExecutorFaderFunction,
}

impl DemexExecutor {
    pub fn new(id: u32, runtime: SequenceRuntime, function: DemexExecutorFaderFunction) -> Self {
        Self {
            id,
            runtime,
            fader_function: function,
            priority: FixtureChannelValuePriority::Ltp,
            value: 0.0,
            stomp_protected: false,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn stomp_protected(&self) -> bool {
        self.stomp_protected
    }

    pub fn display_name(&self, preset_handler: &PresetHandler) -> String {
        let sequence_name = preset_handler
            .get_sequence(self.runtime.sequence_id())
            .map(|seq| seq.name());
        sequence_name.unwrap_or("[Deleted Sequence]").to_string()
    }

    pub fn priority(&self) -> FixtureChannelValuePriority {
        self.priority
    }

    pub fn runtime(&self) -> &SequenceRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut SequenceRuntime {
        &mut self.runtime
    }

    pub fn fader_function(&self) -> &DemexExecutorFaderFunction {
        &self.fader_function
    }

    pub fn fader_function_mut(&mut self) -> &mut DemexExecutorFaderFunction {
        &mut self.fader_function
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn go(
        &mut self,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
        event_list: &mut DemexEventList,
    ) {
        if !self.is_active() {
            self.start(fixture_handler, preset_handler, time_offset, event_list);
            return;
        }

        let (should_stop, events) = self.runtime.next_cue(preset_handler, time_offset);
        event_list.push_all_optional(
            events.map(|events| events.into_iter().map(|event| event.into_event(self.id))),
        );

        if should_stop {
            self.stop(fixture_handler, preset_handler, event_list);
            return;
        }

        event_list.push(DemexEvent::ExecutorGo(self.id))
    }

    fn set_fader_value(&mut self, value: f32, event_list: &mut DemexEventList) {
        self.value = value;
        event_list.push(DemexEvent::ExecutorFaderValueChanged {
            executor_id: self.id,
            value,
        });
    }

    pub fn set_value(
        &mut self,
        value: f32,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
        event_list: &mut DemexEventList,
    ) {
        if value == 0.0 {
            self.stop(fixture_handler, preset_handler, event_list);
            return; // this will set the fader value to 0.0
        }

        if !self.is_active() {
            self.start(fixture_handler, preset_handler, time_offset, event_list);
            return; // this will set the fader value to 1.0
        }

        self.set_fader_value(value, event_list);
    }

    pub fn is_active(&self) -> bool {
        self.runtime.is_started()
    }

    pub fn fixtures(&self, preset_handler: &PresetHandler) -> HashSet<FixturePath> {
        let sequence = preset_handler
            .get_sequence(self.runtime.sequence_id())
            .unwrap();
        sequence.affected_fixtures(preset_handler)
    }

    pub fn start(
        &mut self,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
        event_list: &mut DemexEventList,
    ) {
        self.set_fader_value(1.0, event_list);
        let event = self.runtime.start(time_offset, preset_handler);

        // self.started_at = Some(time::Instant::now() - time::Duration::from_secs_f32(time_offset));

        for fixture_path in self.fixtures(preset_handler) {
            if let Ok(fixture_state) = fixture_handler.fixture_mut(&fixture_path) {
                fixture_state.push_value_source(FixtureChannelValueSource::Executor {
                    executor_id: self.id,
                });
            }
        }

        event_list.push(DemexEvent::ExecutorGo(self.id));
        event_list.push_optional(
            event.map(|event| DemexEvent::ExecutorUpdateEvent { id: self.id, event }),
        );
    }

    pub fn cue_out(&mut self, time_offset: f32) {
        self.runtime.cue_out(time_offset);
    }

    pub fn stop(
        &mut self,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &PresetHandler,
        event_list: &mut DemexEventList,
    ) {
        self.set_fader_value(0.0, event_list);
        self.runtime.stop();

        for fixture_path in self.fixtures(preset_handler) {
            if let Ok(fixture_state) = fixture_handler.fixture_mut(&fixture_path) {
                fixture_state.remove_value_source(FixtureChannelValueSource::Executor {
                    executor_id: self.id,
                });
            }
        }

        event_list.push(DemexEvent::ExecutorStop(self.id));
    }

    pub fn attribute_value(
        &self,
        fixture_path: &FixturePath,
        attribute: &FixtureChannel3Attribute,
        preset_handler: &PresetHandler,
        _timing_handler: &TimingHandler,
    ) -> Result<FadeFixtureChannelValue, FixtureError> {
        if !self.is_active() {
            return Err(FixtureError::GdtfAttributeValueNotFound(*attribute));
        }

        let sequence = preset_handler
            .get_sequence(self.runtime.sequence_id())
            .unwrap();
        let fixtures = sequence.affected_fixtures(preset_handler);

        if !fixtures.contains(fixture_path) {
            return Err(FixtureError::GdtfAttributeValueNotFound(*attribute));
        }

        let _speed_multiplier = if self.fader_function == DemexExecutorFaderFunction::Speed {
            self.value
        } else {
            1.0
        };

        self.runtime
            .attribute_value(fixture_path, attribute, self.priority, preset_handler)
            .map(|value| match &self.fader_function {
                DemexExecutorFaderFunction::FadeAll => value.multiply(self.value),
                DemexExecutorFaderFunction::Intensity => {
                    if matches!(attribute, FixtureChannel3Attribute::Dimmer) {
                        value.multiply(self.value)
                    } else {
                        value
                    }
                }
                DemexExecutorFaderFunction::FadeFeatures(features) => {
                    if attribute
                        .feature_type()
                        .is_some_and(|ft| features.contains(&ft))
                    {
                        value.multiply(self.value)
                    } else {
                        value
                    }
                }
                _ => value,
            })
            .ok_or(FixtureError::GdtfAttributeValueNotFound(*attribute))
    }

    pub fn update(
        &mut self,
        patch: &Patch,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        event_list: &mut DemexEventList,
    ) {
        let (should_stop, events) = self.runtime.update(
            if self.fader_function == DemexExecutorFaderFunction::Speed {
                self.value
            } else {
                1.0
            },
            patch,
            fixture_handler,
            preset_handler,
            timing_handler,
            self.priority,
        );

        event_list.push_all_optional(
            events.map(|events| events.into_iter().map(|event| event.into_event(self.id))),
        );

        if should_stop {
            self.stop(fixture_handler, preset_handler, event_list);
        }
    }
}

#[derive(strum_macros::EnumString, strum_macros::Display)]
pub enum DemexExecutorProperty {
    Priority,
    FaderFunction,
    StompProtected,
}

implement_set_property! {
    for DemexExecutor with DemexExecutorProperty,

    FaderFunction => fader_function as DemexExecutorFaderFunction,
    Priority => priority as FixtureChannelValuePriority,
    StompProtected => stomp_protected as bool
}

impl From<&DemexExecutor> for PoolItem {
    fn from(value: &DemexExecutor) -> Self {
        PoolItem {
            id: value.id,
            name: PoolItemName::reference(PoolType::Sequence, value.runtime.sequence_id()),
            colors: None,
            flags: 0,
        }
    }
}
