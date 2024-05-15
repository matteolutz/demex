use self::{channel::FixtureChannel, error::FixtureError};

pub mod channel;
pub mod error;
pub mod handler;

#[derive(Debug)]
pub struct Fixture {
    id: u32,
    name: String,
    patch: Vec<FixtureChannel>,
    universe: u16,
    start_address: u8,
}

impl Fixture {
    pub fn new(
        id: u32,
        name: String,
        patch: Vec<FixtureChannel>,
        universe: u16,
        start_address: u8,
    ) -> Result<Self, FixtureError> {
        // validate, that the patch is not empty
        if patch.is_empty() {
            return Err(FixtureError::EmptyPatch);
        }

        // check, that each channel type is unique
        let mut channel_types = Vec::new();
        for channel in &patch {
            if channel_types.contains(&channel.name()) {
                return Err(FixtureError::DuplicateChannelType);
            }
            channel_types.push(channel.name());
        }

        Ok(Self {
            id,
            name,
            patch,
            universe,
            start_address,
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn patch(&self) -> &Vec<FixtureChannel> {
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

    pub fn generate_data_packet(&self) -> Vec<u8> {
        self.patch
            .iter()
            .flat_map(|channel| channel.generate_data_packet())
            .collect()
    }
}

impl Fixture {
    pub fn is_home(&self) -> bool {
        self.patch.iter().all(|c| match c {
            FixtureChannel::Intensity(intens) => intens.is_none(),
            FixtureChannel::ColorRGB(rgb) => rgb.is_none(),
            FixtureChannel::PositionPanTilt(pan_tilt) => pan_tilt.is_none(),
            FixtureChannel::Maintenance(_, value) => value.is_none(),
        })
    }

    pub fn intensity(&self) -> Result<Option<u8>, FixtureError> {
        match self
            .patch
            .iter()
            .find(|c| matches!(c, FixtureChannel::Intensity(_)))
        {
            Some(FixtureChannel::Intensity(intens)) => Ok(*intens),
            _ => Err(FixtureError::ChannelNotFound(Some("Intensity".to_string()))),
        }
    }

    pub fn maintenance(&self, name: &str) -> Result<Option<u8>, FixtureError> {
        match self.patch.iter().find(|c| match c {
            FixtureChannel::Maintenance(n, _) => n == name,
            _ => false,
        }) {
            Some(FixtureChannel::Maintenance(_, value)) => Ok(*value),
            _ => Err(FixtureError::ChannelNotFound(Some(name.to_string()))),
        }
    }
}

impl Fixture {
    pub fn home(&mut self) -> Result<(), FixtureError> {
        // set all channels to none

        for channel in self.patch.iter_mut() {
            match channel {
                FixtureChannel::Intensity(intens) => *intens = None,
                FixtureChannel::ColorRGB(rgb) => *rgb = None,
                FixtureChannel::PositionPanTilt(pan_tilt) => *pan_tilt = None,
                FixtureChannel::Maintenance(_, value) => *value = None,
            }
        }

        Ok(())
    }

    pub fn set_intensity(&mut self, intens: u8) -> Result<(), FixtureError> {
        match self
            .patch
            .iter_mut()
            .find(|c| matches!(c, FixtureChannel::Intensity(_)))
        {
            Some(FixtureChannel::Intensity(intens_ref)) => {
                *intens_ref = Some(intens);
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(Some("Intensity".to_string()))),
        }
    }

    pub fn set_maintenance(&mut self, name: &str, value: u8) -> Result<(), FixtureError> {
        match self.patch.iter_mut().find(|c| match c {
            FixtureChannel::Maintenance(n, _) => n == name,
            _ => false,
        }) {
            Some(FixtureChannel::Maintenance(_, value_ref)) => {
                *value_ref = Some(value);
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(Some(name.to_string()))),
        }
    }
}

impl std::fmt::Display for Fixture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut state = String::new();

        if let Ok(intens) = self.intensity() {
            state.push_str(
                format!(
                    "{}%",
                    intens.map(|i| i.to_string()).unwrap_or("-".to_string())
                )
                .as_str(),
            );
        }

        write!(
            f,
            "{}\n{} (U{}.{})\n\n{}",
            self.name, self.id, self.universe, self.start_address, state
        )
    }
}
