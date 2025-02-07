#[derive(Debug)]
pub enum ActionRunResult {
    Default,
    Info(String),
    Warn(String),
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }
}
