use serde::{Deserialize, Serialize};

use crate::ui::{constants::MAX_VIEWPORTS, viewport::DemexViewport};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DemexShowUiConfig {
    /*pub detached_tabs: HashSet<DemexTab>,
    pub detached_tabs_config: HashMap<DemexTab, DetachedTabConfig>,*/
    pub viewports: [DemexViewport; MAX_VIEWPORTS],
}

impl Default for DemexShowUiConfig {
    fn default() -> Self {
        Self {
            viewports: DemexViewport::default_viewports(),
        }
    }
}
