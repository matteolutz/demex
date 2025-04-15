use crate::fixture::gdtf::error::DemexGdtfError;

#[derive(Debug)]
pub enum GdtfFeature {
    Dimmer,
    PanTilt,
    Unknown(GdtfUnknownFeature),
}

impl TryFrom<gdtf::attribute::Feature> for GdtfFeature {
    type Error = DemexGdtfError;

    fn try_from(value: gdtf::attribute::Feature) -> Result<Self, Self::Error> {
        value
            .name
            .ok_or(DemexGdtfError::UnnamedFeature)
            .map(|name| match name.as_ref() {
                "Dimmer" => Self::Dimmer,
                "PanTilt" => Self::PanTilt,
                unknown => Self::Unknown(GdtfUnknownFeature {
                    name: unknown.to_owned(),
                }),
            })
    }
}

#[derive(Debug)]
pub struct GdtfUnknownFeature {
    name: String,
}

impl GdtfUnknownFeature {
    pub fn name(&self) -> &str {
        &self.name
    }
}
