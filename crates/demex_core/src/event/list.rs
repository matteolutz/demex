use std::sync::mpsc;

use crate::{engine::comm::DemexEngineCommEvent, event::DemexEvent};

#[derive(Debug, Clone, Default)]
pub struct DemexEventList {
    events: Vec<DemexEvent>,
}

impl DemexEventList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[DemexEvent] {
        &self.events
    }

    pub fn push(&mut self, event: impl Into<DemexEvent>) {
        self.events.push(event.into());
    }

    pub fn push_optional(&mut self, event: Option<impl Into<DemexEvent>>) {
        if let Some(event) = event {
            self.push(event);
        }
    }

    pub fn push_all(&mut self, events: impl IntoIterator<Item = DemexEvent>) {
        self.events.extend(events);
    }

    pub fn push_all_optional(&mut self, events: Option<impl IntoIterator<Item = DemexEvent>>) {
        if let Some(events) = events {
            self.push_all(events);
        }
    }

    pub fn send(
        &mut self,
        tx: &mpsc::Sender<DemexEngineCommEvent>,
    ) -> Result<(), mpsc::SendError<DemexEngineCommEvent>> {
        for event in self.events.drain(..) {
            tx.send(event.into())?;
        }
        Ok(())
    }
}
