use crate::fixture::timing::timecode::trigger::TimecodeTrigger;

#[derive(Debug, Clone)]
pub struct TimecodeTriggerScheduler {
    triggers_sorted: Vec<TimecodeTrigger>,
    next_trigger_idx: usize,
}

impl TimecodeTriggerScheduler {
    pub fn new(mut triggers: Vec<TimecodeTrigger>) -> Self {
        triggers.sort_by_key(|t| t.millis); // Ensure triggers are sorted by time

        Self {
            triggers_sorted: triggers,
            next_trigger_idx: 0,
        }
    }

    pub fn triggers(&self) -> &[TimecodeTrigger] {
        &self.triggers_sorted
    }

    pub fn add_trigger(&mut self, trigger: TimecodeTrigger) {
        match self
            .triggers_sorted
            .binary_search_by_key(&trigger.millis, |trigger| trigger.millis)
        {
            Ok(idx) | Err(idx) => self.triggers_sorted.insert(idx, trigger),
        }
    }

    pub fn recalculate_next_trigger(&mut self, current_millis: u64) {
        self.next_trigger_idx = self
            .triggers_sorted
            .iter()
            .position(|t| t.millis > current_millis)
            .unwrap_or(self.triggers_sorted.len());
    }

    pub fn update(&mut self, current_millis: u64) -> Vec<&TimecodeTrigger> {
        let mut executed_triggers = Vec::new();

        for (idx, trigger) in self
            .triggers_sorted
            .iter()
            .skip(self.next_trigger_idx)
            .enumerate()
        {
            if trigger.millis > current_millis {
                break;
            }

            executed_triggers.push(trigger);
            self.next_trigger_idx = idx + 1;
        }

        executed_triggers
    }
}
