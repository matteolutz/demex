use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::ui::{tabs::DemexTab, DetachedTabConfig};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DemexShowUiConfig {
    pub detached_tabs: HashSet<DemexTab>,
    pub detached_tabs_config: HashMap<DemexTab, DetachedTabConfig>,
}
