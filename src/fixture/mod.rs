use self::state::FixtureState;

pub mod handler;
pub mod state;

#[derive(Debug)]
pub enum FixtureChannelType {
    Intesity,
    ColorR,
    ColorG,
    ColorB,
    Pan,
    Tilt,
}

#[derive(Debug)]
pub struct Fixture {
    id: u32,
    name: String,
    channel_map: Vec<FixtureChannelType>,
    universe: u16,
    start_address: u8,
}

impl Fixture {
    pub fn new(
        id: u32,
        name: String,
        channel_map: Vec<FixtureChannelType>,
        universe: u16,
        start_address: u8,
    ) -> Self {
        Self {
            id,
            name,
            channel_map,
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

    pub fn channel_map(&self) -> &Vec<FixtureChannelType> {
        &self.channel_map
    }

    pub fn universe(&self) -> u16 {
        self.universe
    }

    pub fn start_address(&self) -> u8 {
        self.start_address
    }

    pub fn generate_data_packet(&self, fixture_state: FixtureState) -> Vec<u8> {
        let mut data_packet = Vec::new();
        for channel in self.channel_map.iter() {
            match channel {
                FixtureChannelType::Intesity => {
                    data_packet.push(fixture_state.intensity());
                }
                FixtureChannelType::ColorR => {
                    data_packet.push(fixture_state.color().0);
                }
                FixtureChannelType::ColorG => {
                    data_packet.push(fixture_state.color().1);
                }
                FixtureChannelType::ColorB => {
                    data_packet.push(fixture_state.color().2);
                }
                FixtureChannelType::Pan => {
                    data_packet.push(fixture_state.pan());
                }
                FixtureChannelType::Tilt => {
                    data_packet.push(fixture_state.tilt());
                }
            }
        }
        data_packet
    }
}
