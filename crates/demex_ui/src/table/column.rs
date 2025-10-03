/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use std::hash::Hash;

use gpui::{Pixels, SharedString, px};

#[derive(Debug, Clone)]
pub struct Column<I: Clone + Eq + Hash> {
    pub id: I,
    pub label: SharedString,
    pub width: Pixels,
}

impl<I: Clone + Eq + Hash> Column<I> {
    pub fn new(id: impl Into<I>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            width: px(100.0),
        }
    }

    pub fn new_auto(id: impl Into<I> + ToString) -> Self {
        let name = id.to_string();
        Self::new(id, name)
    }

    pub fn with_width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }
}
