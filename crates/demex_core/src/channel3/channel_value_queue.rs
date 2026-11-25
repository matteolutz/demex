use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    time,
};

use itertools::Itertools;

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value_discrete::FixtureChannelDiscreteValue,
        clamped_value::ClampedValue,
    },
    engine::component::Component,
    fixture::{Fixture, FixturePath},
};

#[derive(Debug)]
pub struct ChannelValueQueueEntry {
    pub fixture_path: FixturePath,
    pub master_value: ClampedValue,
    pub values:
        HashMap<FixtureChannel3Attribute, (FixtureChannelDiscreteValue, Option<time::Instant>)>,
}

impl ChannelValueQueueEntry {
    pub fn sorted_values(
        self,
        fixture: &Fixture,
    ) -> impl Iterator<Item = (FixtureChannel3Attribute, FixtureChannelDiscreteValue)> {
        self.values
            .into_iter()
            .sorted_by(
                |(attr_a, (value_a, updated_a)), (attr_b, (value_b, updated_b))| {
                    let a_initial = fixture
                        .channel_function(attr_a)
                        .is_some_and(|cf| cf.is_initial());
                    let b_initial = fixture
                        .channel_function(attr_b)
                        .is_some_and(|cf| cf.is_initial());

                    let _a_is_home = value_a.is_home();
                    let _b_is_home = value_b.is_home();

                    match (a_initial, updated_a, b_initial, updated_b) {
                        // is_initial=false, updated=None should be first
                        (false, None, false, None) => Ordering::Equal,
                        (false, None, _, _) => Ordering::Less,
                        (_, _, false, None) => Ordering::Greater,

                        // is_initial=true, updated=None should be second
                        (true, None, true, None) => std::cmp::Ordering::Equal,
                        (true, None, _, _) => std::cmp::Ordering::Less,
                        (_, _, true, None) => std::cmp::Ordering::Greater,

                        // is_initial=? and updated=Some(...) should be last, sorted by updated_at.elapsed() smallest first
                        (_, Some(updated_a), _, Some(updated_b)) => {
                            // We want smallest elapsed to be last, so reverse the comparison
                            // if item1 is 'more elapsed' (further in the future), it should come after item2
                            updated_b.cmp(updated_a)
                        }
                    }
                },
            )
            .map(|(attr, (value, _))| (attr, value))
    }
}

#[derive(Default)]
pub struct ChannelValueQueue {
    inner: VecDeque<ChannelValueQueueEntry>,
}

impl ChannelValueQueue {
    pub fn enqueue(
        &mut self,
        fixture_path: FixturePath,
        master_value: ClampedValue,
        values: HashMap<
            FixtureChannel3Attribute,
            (FixtureChannelDiscreteValue, Option<time::Instant>),
        >,
    ) {
        self.inner.push_back(ChannelValueQueueEntry {
            fixture_path,
            master_value,
            values,
        });
    }

    pub fn inner_mut(&mut self) -> &mut VecDeque<ChannelValueQueueEntry> {
        &mut self.inner
    }
}

impl Component for ChannelValueQueue {}
