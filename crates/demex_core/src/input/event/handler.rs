use crate::{engine::component::Component, event::DemexEvent};

impl Component for DemexInputDeviceEventHandler {}

#[derive(Debug, Default)]
pub struct DemexInputDeviceEventHandler {
    events: Vec<DemexEvent>,
}

impl DemexInputDeviceEventHandler {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn events(&self) -> &[DemexEvent] {
        &self.events
    }

    pub fn push_event(&mut self, event: DemexEvent) {
        self.events.push(event);
    }

    pub fn push_events(&mut self, events: impl Iterator<Item = DemexEvent>) {
        self.events.extend(events);
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }
}
