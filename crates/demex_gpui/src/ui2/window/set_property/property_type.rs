use std::ops::RangeInclusive;

use gpui::Context;
use gpui_component::input::InputState;

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

    Integer {
        range: Option<RangeInclusive<u32>>,
    },
    Float {
        range: Option<RangeInclusive<f32>>,
    },
    Percentage,

    RelativeTime {
        unit: TimeUnit,
        allow_negative: bool,
    },
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

    pub fn relative_time(unit: TimeUnit, allow_negative: bool) -> Self {
        Self::RelativeTime {
            unit,
            allow_negative,
        }
    }

    pub fn relative_positive_seconds() -> Self {
        Self::RelativeTime {
            unit: TimeUnit::Seconds,
            allow_negative: false,
        }
    }
}

impl SetPropertyWindowPropertyType {
    pub fn format_value(&self, value: &str) -> String {
        match self {
            Self::Float { .. } | Self::Integer { .. } | Self::String => value.to_string(),
            Self::Percentage => format!("{}%", value),
            Self::RelativeTime {
                unit,
                allow_negative: _,
            } => format!("{}{}", value, unit.get_suffix()),
        }
    }

    pub fn get_validator(self) -> Box<dyn Fn(&str, &mut Context<InputState>) -> bool + 'static> {
        match self {
            Self::String => Box::new(|_, _| true),
            Self::Float { range } => Box::new(move |value, _| {
                let Some(value) = value.parse::<f32>().ok() else {
                    return false;
                };

                if let Some(range) = &range {
                    return range.contains(&value);
                }

                true
            }),
            Self::Integer { range } => Box::new(move |value, _| {
                let Some(value) = value.parse::<u32>().ok() else {
                    return false;
                };

                if let Some(range) = &range {
                    return range.contains(&value);
                }

                true
            }),
            Self::Percentage => Box::new(move |value, _| {
                let Some(value) = value.parse::<f32>().ok() else {
                    return false;
                };

                (0.0..=100.0).contains(&value)
            }),
            Self::RelativeTime {
                unit: _,
                allow_negative,
            } => Box::new(move |value, _| {
                let Some(value) = value.parse::<f32>().ok() else {
                    return false;
                };

                allow_negative || (!allow_negative && value >= 0.0)
            }),
        }
    }
}
