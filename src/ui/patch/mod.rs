use crate::fixture::patch::FixtureTypeAndMode;

pub mod template;

pub struct PatchUiNewFixture {
    pub id: u32,

    pub universe: u16,
    pub start_address: u16,

    pub type_and_mode: FixtureTypeAndMode,
}
