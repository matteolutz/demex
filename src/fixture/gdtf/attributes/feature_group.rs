use crate::fixture::gdtf::error::DemexGdtfError;

use super::feature::GdtfFeature;

#[derive(Debug)]
pub struct GdtfFeatureGroup {
    name: String,
    pretty: String,
    features: Vec<GdtfFeature>,
}

impl GdtfFeatureGroup {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pretty(&self) -> &str {
        &self.pretty
    }

    pub fn features(&self) -> &[GdtfFeature] {
        &self.features
    }
}

impl TryFrom<gdtf::attribute::FeatureGroup> for GdtfFeatureGroup {
    type Error = DemexGdtfError;

    fn try_from(value: gdtf::attribute::FeatureGroup) -> Result<Self, Self::Error> {
        let name = value
            .name
            .map(|name| name.as_ref().to_owned())
            .ok_or(DemexGdtfError::UnnamedFeatureGroup)?;

        let features: Result<Vec<GdtfFeature>, DemexGdtfError> = value
            .features
            .into_iter()
            .map(GdtfFeature::try_from)
            .collect();

        Ok(Self {
            name,
            pretty: value.pretty,
            features: features?,
        })
    }
}
