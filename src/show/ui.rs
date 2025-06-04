use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ui::{constants::MAX_VIEWPORTS, viewport::DemexViewport};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DemexShowUiConfig {
    pub viewports: [DemexViewport; MAX_VIEWPORTS],

    #[serde(default)]
    pub lock_image: Option<PathBuf>,
}

impl Default for DemexShowUiConfig {
    fn default() -> Self {
        Self {
            viewports: DemexViewport::default_viewports(),
            lock_image: None,
        }
    }
}
