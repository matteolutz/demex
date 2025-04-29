lazy_static::lazy_static! {
    pub static ref ATTRIBUTE_WILDCARD_REGEX: regex::Regex = regex::Regex::new(r"(\d+)").unwrap();
}

#[derive(Debug, Copy, Clone, strum_macros::EnumIter)]
pub enum FixtureChannel3Attribute {}

impl FixtureChannel3Attribute {
    pub fn attribute_matches(fixture_attribute_name: &str, attribute_name: &str) -> bool {
        let fixture_attribute_name =
            ATTRIBUTE_WILDCARD_REGEX.replace(fixture_attribute_name, "(n)");
        let fixture_attribute_name =
            ATTRIBUTE_WILDCARD_REGEX.replace(&fixture_attribute_name, "(m)");

        fixture_attribute_name == attribute_name
    }
}
