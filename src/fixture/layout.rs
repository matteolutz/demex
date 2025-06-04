use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FixtureLayoutDecoration {
    Label {
        pos: emath::Pos2,
        text: String,
        font_size: f32,
    },
    Rect {
        min: emath::Pos2,
        max: emath::Pos2,
        stroke_width: f32,
    },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum FixtureLayoutEntryType {
    Rect,
    Circle,
    Triangle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureLayoutEntry {
    fixture_id: u32,
    // offset to center
    position: emath::Pos2,
    // size
    size: emath::Vec2,
    entry_type: FixtureLayoutEntryType,
}

impl FixtureLayoutEntry {
    pub fn new(
        fixture_id: u32,
        position: emath::Pos2,
        size: emath::Vec2,
        entry_type: FixtureLayoutEntryType,
    ) -> Self {
        Self {
            fixture_id,
            position,
            size,
            entry_type,
        }
    }

    pub fn fixture_id(&self) -> u32 {
        self.fixture_id
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FixtureLayout {
    fixtures: Vec<FixtureLayoutEntry>,
    decorations: Vec<FixtureLayoutDecoration>,
}

impl FixtureLayout {
    pub fn new(fixtures: Vec<FixtureLayoutEntry>) -> Self {
        Self {
            fixtures,
            decorations: Vec::new(),
        }
    }

    pub fn decorations(&self) -> &[FixtureLayoutDecoration] {
        &self.decorations
    }

    pub fn fixtures(&self) -> &[FixtureLayoutEntry] {
        &self.fixtures
    }
}

impl From<Vec<FixtureLayoutEntry>> for FixtureLayout {
    fn from(value: Vec<FixtureLayoutEntry>) -> Self {
        Self::new(value)
    }
}
