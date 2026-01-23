use serde::{Deserialize, Serialize};

use crate::fixture::{FixtureId, FixturePath};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum FixtureLayoutEntryType {
    Rect,
    Circle,
    Triangle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureLayoutEntryLine {
    to_fixture_id: FixtureId,
    offset: emath::Vec2,
}

impl FixtureLayoutEntryLine {
    pub fn to_fixture_id(&self) -> FixtureId {
        self.to_fixture_id
    }

    pub fn offset(&self) -> &emath::Vec2 {
        &self.offset
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureLayoutEntry {
    fixture_path: FixturePath,
    // offset to center
    position: emath::Pos2,
    // size
    size: emath::Vec2,
    entry_type: FixtureLayoutEntryType,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    line: Option<FixtureLayoutEntryLine>,
}

impl FixtureLayoutEntry {
    pub fn new(
        fixture_path: FixturePath,
        position: emath::Pos2,
        size: emath::Vec2,
        entry_type: FixtureLayoutEntryType,
    ) -> Self {
        Self {
            fixture_path,
            position,
            size,
            entry_type,
            line: None,
        }
    }

    pub fn fixture_path(&self) -> &FixturePath {
        &self.fixture_path
    }

    pub fn position(&self) -> &emath::Pos2 {
        &self.position
    }

    pub fn size(&self) -> &emath::Vec2 {
        &self.size
    }

    pub fn entry_type(&self) -> FixtureLayoutEntryType {
        self.entry_type
    }

    pub fn line(&self) -> Option<&FixtureLayoutEntryLine> {
        self.line.as_ref()
    }
}
