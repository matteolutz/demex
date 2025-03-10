use feature_type::FixtureFeatureType;

pub mod feature_config;
pub mod feature_group;
pub mod feature_state;
pub mod feature_type;
pub mod feature_value;
pub mod wheel;

pub trait IntoFeatureType {
    fn feature_type(&self) -> FixtureFeatureType;
}
