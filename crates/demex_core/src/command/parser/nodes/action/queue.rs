use std::{collections::VecDeque, time};

use crate::{
    command::parser::nodes::action::{Action, ActionIssuer, DeferredAction},
    engine::component::Component,
};

#[derive(Default)]
pub struct ActionQueue {
    inner: VecDeque<DeferredAction>,
}

impl ActionQueue {
    pub fn enqueue_now(&mut self, action: Action, issuer: ActionIssuer) {
        self.enqueue_deferred(DeferredAction {
            action,
            issued_at: time::Instant::now(),
            issuer,
        });
    }

    pub fn enqueue_at(&mut self, action: Action, issued_at: time::Instant, issuer: ActionIssuer) {
        self.enqueue_deferred(DeferredAction {
            action,
            issued_at,
            issuer,
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

    pub fn inner_mut(&mut self) -> &mut VecDeque<DeferredAction> {
        &mut self.inner
    }
}

impl Component for ActionQueue {}
