use std::{collections::HashMap, time};

use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel3::channel_value::FixtureChannelValue3,
    gdtf::GdtfFixture,
    handler::{FixtureHandler, FixtureTypeList},
    presets::PresetHandler,
    timing::TimingHandler,
    value_source::FixtureChannelValuePriority,
};

use super::{
    cue::{Cue, CueTrigger},
    FadeFixtureChannelValue, Sequence, SequenceStopBehavior,
};

pub struct ActiveCueData {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SequenceRuntimeState {
    #[default]
    Stopped,

    Cues {
        active_cues: Vec<(usize, time::Instant)>,
        current_cue: usize,
    },
}

impl SequenceRuntimeState {
    pub fn is_started(&self) -> bool {
        match self {
            Self::Cues { .. } => true,
            Self::Stopped => false,
        }
    }

    pub fn start(time_offset: f32) -> Self {
        Self::Cues {
            active_cues: vec![(
                0,
                time::Instant::now() - time::Duration::from_secs_f32(time_offset),
            )],
            current_cue: 0,
        }
    }

    pub fn when_started(&self) -> Option<(&[(usize, time::Instant)], usize, time::Instant)> {
        match self {
            Self::Cues {
                active_cues,
                current_cue,
            } => Some((
                active_cues,
                *current_cue,
                active_cues
                    .iter()
                    .find_map(|(i, activated)| {
                        if *i == *current_cue {
                            Some(*activated)
                        } else {
                            None
                        }
                    })
                    .unwrap(),
            )),
            _ => None,
        }
    }

    pub fn when_started_mut(
        &mut self,
    ) -> Option<(&mut Vec<(usize, time::Instant)>, &mut usize, time::Instant)> {
        match self {
            Self::Cues {
                active_cues,
                current_cue,
            } => {
                let current_cue_activated_at = active_cues
                    .iter()
                    .find_map(|(i, activated)| {
                        if *i == *current_cue {
                            Some(*activated)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                Some((active_cues, current_cue, current_cue_activated_at))
            }
            _ => None,
        }
    }

    pub fn activate_cue(self, cue_idx: usize, activated_at: time::Instant) -> Self {
        match self {
            Self::Stopped => Self::Stopped,
            Self::Cues {
                mut active_cues, ..
            } => {
                active_cues.retain(|(i, _)| *i != cue_idx);
                active_cues.push((cue_idx, activated_at));

                Self::Cues {
                    active_cues,
                    current_cue: cue_idx,
                }
            }
        }
    }

    pub fn next_cue(
        self,
        num_cues: usize,
        stop_behavior: SequenceStopBehavior,
        time_offset: f32,
    ) -> (bool, Self) {
        match self {
            Self::Stopped => (false, Self::start(time_offset)),
            Self::Cues {
                mut active_cues,
                current_cue,
            } => {
                if current_cue == num_cues - 1 && stop_behavior != SequenceStopBehavior::Restart {
                    (false, Self::Stopped)
                } else {
                    let should_clear_tracked_values = (current_cue + 1) >= num_cues;

                    let next_cue = (current_cue + 1) % num_cues;
                    let activated_at =
                        time::Instant::now() - time::Duration::from_secs_f32(time_offset);

                    active_cues.retain(|(i, _)| *i != next_cue);
                    active_cues.push((next_cue, activated_at));

                    (
                        should_clear_tracked_values,
                        Self::Cues {
                            active_cues,
                            current_cue: next_cue,
                        },
                    )
                }
            }
        }
    }

    pub fn current_cue_indices(&self) -> Vec<usize> {
        match self {
            Self::Cues { active_cues, .. } => active_cues.iter().map(|(i, _)| *i).collect(),
            Self::Stopped => vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct SequenceRuntime {
    sequence_id: u32,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[cfg_attr(feature = "ui", egui_probe(skip))]
    state: SequenceRuntimeState,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[cfg_attr(feature = "ui", egui_probe(skip))]
    tracked_values: HashMap<u32, HashMap<String, Vec<(usize, FadeFixtureChannelValue)>>>,
}

impl SequenceRuntime {
    pub fn new(sequence_id: u32) -> Self {
        Self {
            sequence_id,
            state: SequenceRuntimeState::default(),
            tracked_values: HashMap::new(),
        }
    }

    pub fn sequence_id(&self) -> u32 {
        self.sequence_id
    }

    pub fn is_started(&self) -> bool {
        self.state.is_started()
    }

    pub fn current_cues(&self) -> Vec<usize> {
        self.state.current_cue_indices()
    }

    pub fn num_cues(&self, preset_handler: &PresetHandler) -> usize {
        preset_handler
            .get_sequence(self.sequence_id)
            .unwrap()
            .cues()
            .len()
    }

    pub fn channel_value(
        &self,
        fixture: &GdtfFixture,
        channel: &gdtf::dmx_mode::DmxChannel,
        priority: FixtureChannelValuePriority,
    ) -> Option<FadeFixtureChannelValue> {
        self.tracked_values.get(&fixture.id()).and_then(|values| {
            values.iter().find_map(|(value_channel_name, values)| {
                if value_channel_name == channel.name().as_ref() {
                    let mut value = FixtureChannelValue3::Home;
                    for (_, v) in values.iter() {
                        value = FixtureChannelValue3::Mix {
                            a: Box::new(value),
                            b: Box::new(v.value().clone()),
                            mix: v.alpha,
                        };
                    }

                    Some(FadeFixtureChannelValue::new(value, 1.0, priority))
                } else {
                    None
                }
            })
        })
    }

    pub fn update_cue_values<'a>(
        tracked_values: &mut HashMap<u32, HashMap<String, Vec<(usize, FadeFixtureChannelValue)>>>,
        fixtures: impl Iterator<Item = &'a u32>,
        cue_idx: usize,
        cue: &Cue,
        cue_delta: f32,
        cue_activated_at: &time::Instant,
        fixture_types: &FixtureTypeList,
        fixture_handler: &FixtureHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        priority: FixtureChannelValuePriority,
        is_mib: bool,
    ) {
        for fixture_id in fixtures {
            let fixture_cue_delta =
                (cue_delta - cue.offset_for_fixture(*fixture_id, preset_handler)).max(0.0);

            let cue_values = cue.values_for_fixture(
                fixture_handler.fixture_immut(*fixture_id).unwrap(),
                fixture_types,
                preset_handler,
                timing_handler,
                Some(*cue_activated_at),
            );

            let mut fixture_cue_fade = if fixture_cue_delta < cue.in_delay() {
                0.0
            } else {
                ((fixture_cue_delta - cue.in_delay()) / cue.in_fade()).min(1.0)
            };

            fixture_cue_fade = cue.fading_function().apply(fixture_cue_fade);

            if fixture_cue_fade == 0.0 {
                continue;
            }

            for value in cue_values {
                if is_mib {
                    let attribute = fixture_handler
                        .fixture_immut(*fixture_id)
                        .unwrap()
                        .get_channel_attribute(fixture_types, value.channel_name());

                    if attribute.is_ok_and(|attribute| attribute == "Dimmer") {
                        continue;
                    }
                }

                let fixture_values = tracked_values.entry(*fixture_id).or_default();
                let (channel_name, value) = value.into();

                if let Some(existing_values) = fixture_values.get_mut(&channel_name) {
                    if fixture_cue_fade == 0.0 {
                        continue;
                    } else if fixture_cue_fade == 1.0 {
                        if let Some((_, existing_cue_value)) =
                            existing_values.iter_mut().find(|(i, _)| *i == cue_idx)
                        {
                            existing_cue_value.set_alpha(1.0);
                        } else {
                            *existing_values = vec![(
                                cue_idx,
                                FadeFixtureChannelValue::new(value, fixture_cue_fade, priority),
                            )];
                            continue;
                        }

                        existing_values.retain(|(i, _)| *i == cue_idx);
                    } else {
                        let existing_cue_value = existing_values
                            .iter_mut()
                            .find(|(value_cue_idx, _)| *value_cue_idx == cue_idx);

                        if let Some((_, existing_cue_value)) = existing_cue_value {
                            existing_cue_value.set_alpha(fixture_cue_fade);
                        } else {
                            existing_values.push((
                                cue_idx,
                                FadeFixtureChannelValue::new(value, fixture_cue_fade, priority),
                            ));
                        }

                        /*
                        existing_values.retain(|(value_cue_idx, _)| *value_cue_idx != cue_idx);
                        existing_values.push((
                            cue_idx,
                            FadeFixtureChannelValue::new(value, fixture_cue_fade, priority),
                        ));
                        */
                    }
                } else {
                    fixture_values.insert(
                        channel_name,
                        vec![(
                            cue_idx,
                            FadeFixtureChannelValue::new(value, fixture_cue_fade, priority),
                        )],
                    );
                }
            }
        }
    }

    pub fn update_values(
        tracked_values: &mut HashMap<u32, HashMap<String, Vec<(usize, FadeFixtureChannelValue)>>>,
        sequence: &Sequence,
        active_cues: &[(usize, time::Instant)],
        current_cue_idx: usize,
        next_cue_idx: Option<usize>,
        fixture_handler: &FixtureHandler,
        fixture_types: &FixtureTypeList,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        priority: FixtureChannelValuePriority,
    ) {
        for (cue_idx, cue_activated_at) in active_cues.iter() {
            let cue = sequence.cue(*cue_idx);

            if cue.block() {
                tracked_values.clear();
            }

            let cue_delta = time::Instant::now()
                .duration_since(*cue_activated_at)
                .as_secs_f32();

            let cue_affected_fixtures = cue.affected_fixtures(preset_handler);

            Self::update_cue_values(
                tracked_values,
                cue_affected_fixtures.iter(),
                *cue_idx,
                cue,
                cue_delta,
                cue_activated_at,
                fixture_types,
                fixture_handler,
                preset_handler,
                timing_handler,
                priority,
                false,
            );

            if *cue_idx == current_cue_idx && cue.move_in_black() && next_cue_idx.is_some() {
                let next_cue_idx = next_cue_idx.unwrap();
                let next_cue = sequence.cue(next_cue_idx);

                let mut next_cue_affected_fixtures = next_cue.affected_fixtures(preset_handler);
                next_cue_affected_fixtures.retain(|f| !cue_affected_fixtures.contains(f));

                Self::update_cue_values(
                    tracked_values,
                    next_cue_affected_fixtures.iter(),
                    next_cue_idx,
                    next_cue,
                    cue_delta,
                    cue_activated_at,
                    fixture_types,
                    fixture_handler,
                    preset_handler,
                    timing_handler,
                    priority,
                    true,
                );
            }
        }
    }

    pub fn update(
        &mut self,
        _speed_multiplier: f32,
        fixture_types: &FixtureTypeList,
        fixture_handler: &FixtureHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        priority: FixtureChannelValuePriority,
    ) -> bool {
        if let Some((active_cues, current_cue_idx, current_cue_activated_at)) =
            self.state.when_started_mut()
        {
            let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

            if sequence.cues().is_empty() {
                return true;
            }

            let current_cue = sequence.cue(*current_cue_idx);

            active_cues.retain(|(cue_idx, cue_activated_at)| {
                let cue = sequence.cue(*cue_idx);
                let cue_in_time = cue.in_time(preset_handler);

                let cue_delta = time::Instant::now()
                    .duration_since(*cue_activated_at)
                    .as_secs_f32();

                cue_delta <= cue_in_time || *cue_idx == *current_cue_idx
            });

            if let Some(next_cue_idx) = Self::next_cue_idx(sequence, *current_cue_idx) {
                let next_cue = sequence.cue(next_cue_idx);

                let should_activate = match next_cue.trigger() {
                    CueTrigger::Time(time) => {
                        current_cue_activated_at.elapsed().as_secs_f32() >= *time
                    }
                    CueTrigger::Follow => {
                        current_cue_activated_at.elapsed().as_secs_f32()
                            >= current_cue.in_time(preset_handler)
                    }
                    CueTrigger::Manual => false,
                };

                if should_activate {
                    active_cues.retain(|(i, _)| *i != next_cue_idx);
                    active_cues.push((next_cue_idx, time::Instant::now()));
                    *current_cue_idx = next_cue_idx;
                }
            }

            Self::update_values(
                &mut self.tracked_values,
                sequence,
                active_cues,
                *current_cue_idx,
                Self::next_cue_idx(sequence, *current_cue_idx),
                fixture_handler,
                fixture_types,
                preset_handler,
                timing_handler,
                priority,
            );

            active_cues.is_empty()
        } else {
            true
        }
    }

    pub fn start(&mut self, time_offset: f32) {
        self.state = SequenceRuntimeState::start(time_offset);
    }

    pub fn stop(&mut self) {
        self.state = SequenceRuntimeState::Stopped;
        self.tracked_values.clear();
    }

    pub fn should_auto_restart(&self, preset_handler: &PresetHandler) -> bool {
        preset_handler
            .get_sequence(self.sequence_id)
            .unwrap()
            .cues()
            .first()
            .map(|c| *c.trigger() == CueTrigger::Follow)
            .unwrap_or(false)
    }

    pub fn next_cue(&mut self, preset_handler: &PresetHandler, time_offset: f32) -> bool {
        let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

        if sequence.cues().is_empty() {
            return true;
        }

        let (should_clear_tracked_values, new_state) = self.state.clone().next_cue(
            sequence.cues().len(),
            sequence.stop_behavior(),
            time_offset,
        );

        if should_clear_tracked_values {
            self.tracked_values.clear();
        }

        self.state = new_state;

        self.state == SequenceRuntimeState::Stopped
    }

    fn next_cue_idx(sequence: &Sequence, current_cue_idx: usize) -> Option<usize> {
        if current_cue_idx == sequence.cues().len() - 1 {
            if sequence.stop_behavior() == SequenceStopBehavior::Restart {
                Some(0)
            } else {
                None
            }
        } else {
            Some(current_cue_idx + 1)
        }
    }

    /*
    pub fn previous_cue_idx(&self, preset_handler: &PresetHandler) -> Option<usize> {
        if let Some((_, _, cue_idx, is_first_cue)) = self.state.when_started() {
            let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

            if cue_idx == 0 {
                // if this is the first cue, we shouldn't return any
                // previous cue. This would distort the fade in time
                // of the first cue
                if !is_first_cue
                    && (self.should_auto_restart(preset_handler)
                        || sequence.stop_behavior() == SequenceStopBehavior::Restart)
                {
                    Some(sequence.cues().len() - 1)
                } else {
                    None
                }
            } else {
                Some(cue_idx - 1)
            }
        } else {
            None
        }
    }

    pub fn next_cue_idx(&self, preset_handler: &PresetHandler) -> Option<usize> {
        if let Some((_, _, cue_idx, _)) = self.state.when_started() {
            let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

            if cue_idx == sequence.cues().len() - 1 {
                if self.should_auto_restart(preset_handler) {
                    Some(0)
                } else {
                    None
                }
            } else {
                Some(cue_idx + 1)
            }
        } else {
            None
        }
    }

    */
}
