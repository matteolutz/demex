use demex_core::show::DemexShow;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct DemexUiShow {
    pub(crate) engine: DemexShow,
}
