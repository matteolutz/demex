use strum::EnumIter;

pub mod demex;
pub mod lee;

#[macro_export]
macro_rules! color_gel {
    ($name:literal, $color:expr) => {
        ColorGel {
            name: $name,
            color: $color,
        }
    };
}

pub struct ColorGel {
    name: &'static str,
    color: [f32; 3],
}

impl ColorGel {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn color(&self) -> [f32; 3] {
        self.color
    }

    pub fn ecolor(&self) -> ecolor::Color32 {
        ecolor::Color32::from_rgb(
            (self.color[0] * 255.0) as u8,
            (self.color[1] * 255.0) as u8,
            (self.color[2] * 255.0) as u8,
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum ColorGelType {
    Demex,
    Lee,
}

impl std::fmt::Display for ColorGelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorGelType::Demex => write!(f, "demex"),
            ColorGelType::Lee => write!(f, "Lee"),
        }
    }
}

impl ColorGelType {
    pub fn gels(&self) -> &[ColorGel] {
        match self {
            Self::Lee => lee::LEE_COLOR_GELS,
            Self::Demex => demex::DEMEX_COLOR_GELS,
        }
    }
}
