use std::{collections::HashMap, time};

use serde::{Deserialize, Serialize};

use crate::{
    channel3::{attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3},
    event::DemexExecutorUpdateEvent,
    fixture::FixturePath,
    patch::Patch,
    presets::PresetHandler,
    state::fixture_state_handler::FixtureStateHandler,
    timing::TimingHandler,
    value_source::FixtureChannelValuePriority,
};

use super::{
    FadeFixtureChannelValue, Sequence, SequenceStopBehavior,
    cue::{Cue, CueTrigger},
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

    CueOut {
        cue_out_started: time::Instant,
    },
}

impl SequenceRuntimeState {
    pub fn is_started(&self) -> bool {
        match self {
            Self::Cues { .. } => true,
            Self::CueOut { .. } => true,
            Self::Stopped => false,
        }
    }

    pub fn start(at: time::Instant) -> Self {
        Self::Cues {
            active_cues: vec![(0, at)],
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

    pub fn when_cue_out(&self) -> Option<time::Instant> {
        match self {
            Self::CueOut {
                cue_out_started, ..
            } => Some(*cue_out_started),
            _ => None,
        }
    }

    pub fn is_cue_out_done(&self, sequence: &Sequence) -> bool {
        match self {
            Self::CueOut { cue_out_started } => {
                let elapsed_time = time::Instant::now() - *cue_out_started;
                elapsed_time.as_secs_f32() >= sequence.cue_out().fade
            }
            _ => false,
        }
    }

    pub fn activate_cue(self, cue_idx: usize, activated_at: time::Instant) -> Self {
        match self {
            Self::Stopped => Self::Stopped,
            Self::CueOut { .. } => self,
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
        activated_at: time::Instant,
    ) -> (bool, Self) {
        match self {
            Self::Stopped => (false, Self::start(activated_at)),
            Self::CueOut { .. } => (false, Self::Stopped),
            Self::Cues {
                mut active_cues,
                current_cue,
            } => {
                if current_cue == num_cues - 1 && stop_behavior != SequenceStopBehavior::Restart {
                    (
                        false,
                        Self::CueOut {
                            cue_out_started: activated_at,
                        },
                    )
                } else {
                    let should_clear_tracked_values = (current_cue + 1) >= num_cues;

                    let next_cue = (current_cue + 1) % num_cues;

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
            Self::Stopped | Self::CueOut { .. } => vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SequenceRuntime {
    sequence_id: u32,

    #[serde(default, skip_serializing, skip_deserializing)]
    state: SequenceRuntimeState,

    #[serde(default, skip_serializing, skip_deserializing)]
    tracked_values: HashMap<
        FixturePath,
        HashMap<FixtureChannel3Attribute, Vec<(usize, FadeFixtureChannelValue)>>,
    >,
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

    pub fn attribute_value(
        &self,
        fixture_path: &FixturePath,
        attribute: &FixtureChannel3Attribute,
        priority: FixtureChannelValuePriority,
        preset_handler: &PresetHandler,
    ) -> Option<FadeFixtureChannelValue> {
        let tracked_value = self.tracked_values.get(fixture_path).and_then(|values| {
            values.iter().find_map(|(value_attribute, values)| {
                if value_attribute == attribute {
                    let mut value = FixtureChannelValue3::home();
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
        });

        tracked_value.and_then(|tracked_value| {
            if let Some(cue_out_started) = self.state.when_cue_out() {
                let sequence = preset_handler.get_sequence(self.sequence_id).ok()?; // return None, when sequence is not found
                let cue_out_delta = time::Instant::now()
                    .duration_since(cue_out_started)
                    .as_secs_f32();

                let mut cue_out_fade = (cue_out_delta / sequence.cue_out().fade).min(1.0);
                cue_out_fade = sequence.cue_out().fading_function.apply(cue_out_fade);

                Some(FadeFixtureChannelValue::new(
                    FixtureChannelValue3::Mix {
                        a: Box::new(tracked_value.value().clone()),
                        b: Box::new(FixtureChannelValue3::home()),
                        mix: cue_out_fade,
                    },
                    1.0,
                    priority,
                ))
            } else {
                Some(tracked_value)
            }
        })
    }

    pub fn update_cue_values<'a>(
        tracked_values: &mut HashMap<
            FixturePath,
            HashMap<FixtureChannel3Attribute, Vec<(usize, FadeFixtureChannelValue)>>,
        >,
        fixtures: impl Iterator<Item = &'a FixturePath>,
        cue_idx: usize,
        cue: &Cue,
        cue_delta: f32,
        cue_activated_at: &time::Instant,
        patch: &Patch,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        priority: FixtureChannelValuePriority,
        is_mib: bool,
    ) {
        for fixture_path in fixtures {
            let fixture_cue_delta =
                (cue_delta - cue.offset_for_fixture(fixture_path, preset_handler)).max(0.0);

            let cue_values = cue.values_for_fixture(
                patch.fixture(fixture_path).unwrap(),
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
                if is_mib && value.attribute() == &FixtureChannel3Attribute::Dimmer {
                    continue;
                }

                let fixture_values = tracked_values.entry(*fixture_path).or_default();
                let (attribute, value) = value.into();

                if let Some(existing_values) = fixture_values.get_mut(&attribute) {
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
                        attribute,
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
        tracked_values: &mut HashMap<
            FixturePath,
            HashMap<FixtureChannel3Attribute, Vec<(usize, FadeFixtureChannelValue)>>,
        >,
        sequence: &Sequence,
        active_cues: &[(usize, time::Instant)],
        current_cue_idx: usize,
        next_cue_idx: Option<usize>,
        _fixture_handler: &FixtureStateHandler,
        patch: &Patch,
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
                patch,
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
                    patch,
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
        patch: &Patch,
        fixture_handler: &FixtureStateHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        priority: FixtureChannelValuePriority,
    ) -> (bool, Option<Vec<DemexExecutorUpdateEvent>>) {
        if let Some((active_cues, current_cue_idx, current_cue_activated_at)) =
            self.state.when_started_mut()
        {
            let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

            if sequence.cues().is_empty() {
                return (true, None);
            }

            let mut events = vec![];
            let current_cue = sequence.cue(*current_cue_idx);

            active_cues.retain(|(cue_idx, cue_activated_at)| {
                let cue = sequence.cue(*cue_idx);
                let cue_in_time = cue.in_time(preset_handler);

                let cue_delta = time::Instant::now()
                    .duration_since(*cue_activated_at)
                    .as_secs_f32();

                let retain = cue_delta <= cue_in_time || *cue_idx == *current_cue_idx;

                if !retain {
                    events.push(DemexExecutorUpdateEvent::CueDeactivate(cue.cue_idx()));
                }

                retain
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
                    let now = time::Instant::now();
                    active_cues.retain(|(i, _)| *i != next_cue_idx);
                    active_cues.push((next_cue_idx, now));
                    *current_cue_idx = next_cue_idx;

                    events.push(DemexExecutorUpdateEvent::CueActivate(next_cue.cue_idx, now));
                }
            }

            Self::update_values(
                &mut self.tracked_values,
                sequence,
                active_cues,
                *current_cue_idx,
                Self::next_cue_idx(sequence, *current_cue_idx),
                fixture_handler,
                patch,
                preset_handler,
                timing_handler,
                priority,
            );

            (active_cues.is_empty(), Some(events))
        } else {
            (
                self.state
                    .is_cue_out_done(preset_handler.get_sequence(self.sequence_id).unwrap()),
                None,
            )
        }
    }

    pub fn start(
        &mut self,
        time_offset: f32,
        preset_handler: &PresetHandler,
    ) -> Option<DemexExecutorUpdateEvent> {
        let Some(first_cue) = preset_handler
            .get_sequence(self.sequence_id)
            .ok()
            .and_then(|s| s.cues.first())
        else {
            return None;
        };

        let started_at = time::Instant::now() - time::Duration::from_secs_f32(time_offset);

        self.state = SequenceRuntimeState::start(started_at);
        Some(DemexExecutorUpdateEvent::CueActivate(
            first_cue.cue_idx,
            started_at,
        ))
    }

    pub fn cue_out(&mut self, time_offset: f32) {
        if !self.state.is_started() {
            return;
        }

        self.state = SequenceRuntimeState::CueOut {
            cue_out_started: time::Instant::now() - time::Duration::from_secs_f32(time_offset),
        };
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

    pub fn next_cue(
        &mut self,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) -> (bool, Option<Vec<DemexExecutorUpdateEvent>>) {
        let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

        if sequence.cues().is_empty() {
            return (true, None);
        }

        let started_at = time::Instant::now() - time::Duration::from_secs_f32(time_offset);

        let (should_clear_tracked_values, new_state) = self.state.clone().next_cue(
            sequence.cues().len(),
            sequence.stop_behavior(),
            started_at,
        );

        if should_clear_tracked_values {
            self.tracked_values.clear();
        }

        self.state = new_state;

        (self.state == SequenceRuntimeState::Stopped, None)
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
