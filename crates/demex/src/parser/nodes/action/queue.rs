use std::{collections::VecDeque, time};

use crate::parser::nodes::action::{Action, DeferredAction};

#[derive(Default)]
pub struct ActionQueue {
    inner: VecDeque<DeferredAction>,
}

impl ActionQueue {
    pub fn enqueue_now(&mut self, action: Action) {
        self.enqueue_deferred(DeferredAction {
            action,
            issued_at: time::Instant::now(),
        });
    }

    pub fn enqueue_deferred(&mut self, action: DeferredAction) {
        self.inner.push_back(action);
    }

    pub fn dequeue(&mut self) -> Option<DeferredAction> {
        self.inner.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
