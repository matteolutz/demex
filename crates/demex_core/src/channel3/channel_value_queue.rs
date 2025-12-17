use std::collections::{HashMap, VecDeque};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value_discrete::FixtureChannelDiscreteValue,
    },
    engine::component::Component,
    fixture::FixturePath,
};

#[derive(Debug)]
pub struct ChannelValueQueueEntry {
    pub fixture_path: FixturePath,
    pub values: HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>,
}

#[derive(Default)]
pub struct ChannelValueQueue {
    inner: VecDeque<ChannelValueQueueEntry>,
}

impl ChannelValueQueue {
    pub fn enqueue(
        &mut self,
        fixture_path: FixturePath,
        values: HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>,
    ) {
        self.inner.push_back(ChannelValueQueueEntry {
            fixture_path,
            values,
        });
    }

    pub fn inner_mut(&mut self) -> &mut VecDeque<ChannelValueQueueEntry> {
        &mut self.inner
    }
}

impl Component for ChannelValueQueue {}
