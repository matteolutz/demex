pub mod activation_group;
pub mod feature;
pub mod feature_group;

#[derive(Debug)]
pub struct GdtfAttribute {
    name: String,
    pretty: String,

    activation_group: Option<String>,
    feature: String,
    main_attribute: Option<String>,
    physical_unit: gdtf::attribute::PhysicalUnit,
    color: Option<gdtf::values::ColorCie>,
}
