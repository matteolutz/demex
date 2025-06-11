#[derive(Debug, Copy, Clone, Default)]
pub struct FixtureChannelValue3State {
    pub was_updated: bool,
}

impl FixtureChannelValue3State {
    pub fn new_changed() -> Self {
        Self { was_updated: true }
    }

    pub fn reset(&mut self) {
        self.was_updated = false;
    }

    pub fn update(&mut self) {
        self.was_updated = true;
    }
}
