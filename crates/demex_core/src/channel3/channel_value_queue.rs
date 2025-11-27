use std::collections::VecDeque;

use crate::{
    channel3::channel_value_discrete::FixtureChannelDiscreteValue, engine::component::Component,
};

pub struct ChannelValueQueueEntry {
    fixture_id: u32,
    channel: String,
    value: FixtureChannelDiscreteValue,
}

#[derive(Default)]
pub struct ChannelValueQueue {
    inner: VecDeque<ChannelValueQueueEntry>,
}

impl ChannelValueQueue {
    pub fn inner_mut(&mut self) -> &mut VecDeque<ChannelValueQueueEntry> {
        &mut self.inner
    }
}

impl Component for ChannelValueQueue {}
