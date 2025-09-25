use serde::{Deserialize, Serialize};

use crate::ui::{constants::MAX_VIEWPORTS, viewport::DemexViewport};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct DemexShowUiConfig {
    #[cfg_attr(feature = "ui", egui_probe(skip))]
    pub viewports: [DemexViewport; MAX_VIEWPORTS],

    #[serde(default)]
    pub lock_image: Option<String>,
}

impl Default for DemexShowUiConfig {
    fn default() -> Self {
        Self {
            viewports: DemexViewport::default_viewports(),
            lock_image: None,
        }
    }
}
