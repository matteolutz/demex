use std::ops::RangeInclusive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimeUnit {
    Seconds,
}

impl TimeUnit {
    pub fn get_suffix(&self) -> &str {
        match self {
            Self::Seconds => "s",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetPropertyWindowPropertyType {
    String,

    Integer { range: Option<RangeInclusive<u32>> },
    Float { range: Option<RangeInclusive<f32>> },
    Percentage,

    RelativeTime { unit: TimeUnit },
}

impl SetPropertyWindowPropertyType {
    pub fn string() -> Self {
        Self::String
    }

    pub fn integer() -> Self {
        Self::Integer { range: None }
    }

    pub fn integer_with_range(range: impl Into<RangeInclusive<u32>>) -> Self {
        Self::Integer {
            range: Some(range.into()),
        }
    }

    pub fn float() -> Self {
        Self::Float { range: None }
    }

    pub fn float_with_range(range: impl Into<RangeInclusive<f32>>) -> Self {
        Self::Float {
            range: Some(range.into()),
        }
    }

    pub fn relative_time(unit: TimeUnit) -> Self {
        Self::RelativeTime { unit }
    }

    pub fn relative_seconds() -> Self {
        Self::RelativeTime {
            unit: TimeUnit::Seconds,
        }
    }
}

impl SetPropertyWindowPropertyType {
    pub fn format_value(&self, value: &str) -> String {
        match self {
            Self::Float { .. } | Self::Integer { .. } | Self::String => value.to_string(),
            Self::Percentage => format!("{}%", value),
            Self::RelativeTime { unit } => format!("{}{}", value, unit.get_suffix()),
        }
    }
}
