use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub enum DemexViewportPositonState {
    #[default]
    Unset,

    Initial(egui::Rect),
    Rendered(egui::Rect),
}

impl Serialize for DemexViewportPositonState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let option = match self {
            DemexViewportPositonState::Unset => None,
            DemexViewportPositonState::Initial(rect) => Some(rect),
            DemexViewportPositonState::Rendered(rect) => Some(rect),
        };

        // Only serialize the rectangle if it is not Unset
        // we don't care about the Initial or Rendered state, but just the rectangle
        option.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DemexViewportPositonState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let option: Option<egui::Rect> = Option::deserialize(deserializer)?;
        Ok(match option {
            Some(rect) => DemexViewportPositonState::Initial(rect),
            None => DemexViewportPositonState::Unset,
        })
    }
}
