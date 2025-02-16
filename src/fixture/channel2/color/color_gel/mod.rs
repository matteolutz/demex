use lee::LeeColorGelIndex;
use serde::{Deserialize, Serialize};

pub mod lee;

pub trait ColorGelTrait {
    fn get_rgb(&self) -> [f32; 3];
    fn get_name(&self) -> &str;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ColorGel {
    Lee(LeeColorGelIndex),
    Custom { rgb: [f32; 3], name: String },
}

impl ColorGelTrait for ColorGel {
    fn get_rgb(&self) -> [f32; 3] {
        match self {
            Self::Lee(idx) => idx.get_rgb(),
            Self::Custom { rgb, .. } => *rgb,
        }
    }

    fn get_name(&self) -> &str {
        match self {
            Self::Lee(idx) => idx.get_name(),
            Self::Custom { name, .. } => name.as_str(),
        }
    }
}

impl std::fmt::Display for ColorGel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lee(idx) => write!(f, "Lee.{}/{:?}", idx.get_name(), idx),
            Self::Custom { name, .. } => write!(f, "User.{}", name),
        }
    }
}
