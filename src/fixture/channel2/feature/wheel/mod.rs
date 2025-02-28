use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WheelFeatureConfigManualRange {
    pub start: u8,
    pub end: u8,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WheelFeatureConfig<T> {
    macros: Vec<((u8, u8), T)>,
    manual_ranges: Vec<WheelFeatureConfigManualRange>,
}

impl<T> WheelFeatureConfig<T> {
    pub fn macros(&self) -> impl Iterator<Item = &((u8, u8), T)> {
        self.macros.iter()
    }

    pub fn manual_ranges(&self) -> impl Iterator<Item = &WheelFeatureConfigManualRange> {
        self.manual_ranges.iter()
    }

    pub fn try_get_as_macro(&self, value: &WheelFeatureValue) -> Option<&T> {
        match value {
            WheelFeatureValue::Macro(idx) => self.macros.get(*idx).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn to_value(&self, value: &WheelFeatureValue) -> Option<u8> {
        match value {
            WheelFeatureValue::Macro(idx) => self.macros.get(*idx).map(|((min, _), _)| *min),
            WheelFeatureValue::ManualRange { idx, value } => {
                self.manual_ranges.get(*idx).map(|range| {
                    let range = range.start..range.end;
                    let value =
                        range.start as f32 + (range.end as f32 - range.start as f32) * value;
                    value.round() as u8
                })
            }
        }
    }

    pub fn from_value(&self, value: u8) -> Option<WheelFeatureValue> {
        if let Some((idx, _)) = self
            .macros
            .iter()
            .enumerate()
            .find(|(_, ((min, max), _))| (*min..=*max).contains(&value))
        {
            Some(WheelFeatureValue::Macro(idx))
        } else if let Some((idx, range)) = self
            .manual_ranges
            .iter()
            .enumerate()
            .find(|(_, range)| value >= range.start && value <= range.end)
        {
            let range = range.start..range.end;
            let value =
                (value as f32 - range.start as f32) / (range.end as f32 - range.start as f32);
            Some(WheelFeatureValue::ManualRange { idx, value })
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WheelFeatureValue {
    /// A macro (usize = index in the macro list)
    Macro(usize),

    /// A manual range (idx = index in the manual range list, value = 0.0..1.0, which is mapped to the range)
    ManualRange { idx: usize, value: f32 },
}

impl std::fmt::Display for WheelFeatureValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Macro(idx) => write!(f, "Macro {}", idx),
            Self::ManualRange { idx, value } => write!(f, "{} ({:.0}%)", idx, value * 100.0),
        }
    }
}
