use serde::{Deserialize, Serialize};

use super::ColorGelTrait;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LeeColorGelIndex {
    Lee26,
    Lee29,
    Lee46,
}

// This data is taken from https://astera-led.com/wp-content/uploads/DMX-Values-for-common-Colors.pdf
impl ColorGelTrait for LeeColorGelIndex {
    fn get_rgb(&self) -> [f32; 3] {
        match self {
            Self::Lee26 => [1.0, 0.0, 0.071],
            Self::Lee29 => [1.0, 0.153, 0.027],
            Self::Lee46 => [1.0, 0.0, 0.333],
        }
    }

    fn get_name(&self) -> &str {
        match self {
            Self::Lee26 => "Bright Red",
            Self::Lee29 => "Plasa Red",
            Self::Lee46 => "Dark Magenta",
        }
    }
}
