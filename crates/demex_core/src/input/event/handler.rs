use crate::{engine::component::Component, input::event::DemexInputDeviceEvent};

impl Component for DemexInputDeviceEventHandler {}

#[derive(Debug, Default)]
pub struct DemexInputDeviceEventHandler {
    events: Vec<DemexInputDeviceEvent>,
}

impl DemexInputDeviceEventHandler {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn events(&self) -> &[DemexInputDeviceEvent] {
        &self.events
    }

    pub fn push_event(&mut self, event: DemexInputDeviceEvent) {
        self.events.push(event);
    }

    pub fn push_events(&mut self, events: impl Iterator<Item = DemexInputDeviceEvent>) {
        self.events.extend(events);
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }
}
