use std::collections::{HashMap, VecDeque};

use crate::{
    channel3::channel_value_discrete::FixtureChannelDiscreteValue, engine::component::Component,
};

#[derive(Debug)]
pub struct ChannelValueQueueEntry {
    pub fixture_id: u32,
    pub values: HashMap<String, FixtureChannelDiscreteValue>,
}

#[derive(Default)]
pub struct ChannelValueQueue {
    inner: VecDeque<ChannelValueQueueEntry>,
}

impl ChannelValueQueue {
    pub fn enqueue(
        &mut self,
        fixture_id: u32,
        values: HashMap<String, FixtureChannelDiscreteValue>,
    ) {
        self.inner
            .push_back(ChannelValueQueueEntry { fixture_id, values });
    }

    pub fn inner_mut(&mut self) -> &mut VecDeque<ChannelValueQueueEntry> {
        &mut self.inner
    }
}

impl Component for ChannelValueQueue {}
