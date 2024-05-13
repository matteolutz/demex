use self::state::FixtureState;

pub mod handler;
pub mod state;

#[derive(Debug)]
pub enum FixturePatchPart {
    Intesity,
    ColorRGB,
    PositionPanTilt,
}

impl FixturePatchPart {
    pub fn address_bandwidth(&self) -> u8 {
        match self {
            FixturePatchPart::Intesity => 1,
            FixturePatchPart::ColorRGB => 3,
            FixturePatchPart::PositionPanTilt => 2,
        }
    }
}

#[derive(Debug)]
pub struct Fixture {
    id: u32,
    name: String,
    patch: Vec<FixturePatchPart>,
    universe: u16,
    start_address: u8,
}

impl Fixture {
    pub fn new(
        id: u32,
        name: String,
        patch: Vec<FixturePatchPart>,
        universe: u16,
        start_address: u8,
    ) -> Self {
        Self {
            id,
            name,
            patch,
            universe,
            start_address,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn patch(&self) -> &Vec<FixturePatchPart> {
        &self.patch
    }

    pub fn universe(&self) -> u16 {
        self.universe
    }

    pub fn start_address(&self) -> u8 {
        self.start_address
    }

    pub fn address_bandwidth(&self) -> u8 {
        self.patch
            .iter()
            .fold(0u8, |sum, patch_part| sum + patch_part.address_bandwidth())
    }

    pub fn generate_data_packet(&self, fixture_state: FixtureState) -> Vec<u8> {
        self.patch
            .iter()
            .flat_map(|channel| match channel {
                FixturePatchPart::Intesity => vec![fixture_state.intensity().unwrap_or(0)],
                FixturePatchPart::ColorRGB => vec![0, 0, 0],
                _ => vec![0],
            })
            .collect()
    }
}
