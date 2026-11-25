use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{functions::FunctionDelegate, result::ActionRunResult},
    fixture::GdtfFixturePatch,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchFixturesArgs {
    pub fixtures: Vec<GdtfFixturePatch>,
}

impl FunctionDelegate for PatchFixturesArgs {
    fn run(
        &self,
        args: crate::command::parser::nodes::action::ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        let patch = args.patch.clone().add_fixtures(self.fixtures.clone());
        Ok(ActionRunResult::UpdatePatch(patch))
    }
}
