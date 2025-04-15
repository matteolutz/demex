use attributes::{activation_group::GdtfActivationGroup, feature_group::GdtfFeatureGroup};

pub mod attributes;
pub mod error;

#[derive(Debug)]
pub struct GdtfFixture {
    id: u32,
    name: String,

    activation_groups: Vec<GdtfActivationGroup>,
    feature_groups: Vec<GdtfFeatureGroup>,
}
