use strum::IntoEnumIterator;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumIter)]
pub enum AttributeValueDisplayMode {
    #[default]
    Decimal,

    Percentage,
    Bit8,
    Bit16,
}

impl AttributeValueDisplayMode {
    pub fn format_value(&self, value: f32) -> String {
        match self {
            Self::Decimal => format!("{:.2}", value),
            Self::Percentage => format!("{:.2}%", value * 100.0),
            Self::Bit8 => format!("{:.0}", value * u8::MAX as f32),
            Self::Bit16 => format!("{:.0}", value * u16::MAX as f32),
        }
    }

    pub fn next(&self) -> Self {
        let mut iter = Self::iter().cycle();
        iter.find(|mode| mode == self)
            .unwrap_or_else(|| iter.next().unwrap());
        iter.next().unwrap()
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Decimal => ".2",
            Self::Percentage => "%",
            Self::Bit8 => "8b",
            Self::Bit16 => "16b",
        }
    }
}
