#[derive(Debug)]
pub struct ActionRunResult {
    should_update: bool,
}

impl ActionRunResult {
    pub fn new(should_update: bool) -> Self {
        ActionRunResult { should_update }
    }

    pub fn should_update(&self) -> bool {
        self.should_update
    }
}
