#[derive(Debug)]
pub struct GdtfActivationGroup {
    name: String,
}

impl GdtfActivationGroup {
    pub fn name(&self) -> &str {
        &self.name
    }
}
