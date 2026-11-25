use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{
        error::ActionRunError, functions::FunctionDelegate, result::ActionRunResult,
    },
    event::DemexEvent,
    keyframe_effect::{effect_preset::KeyframeEffectPreset, effect_runtime::KeyframeEffectRuntime},
    presets::{
        error::PresetHandlerError,
        preset::{FixturePresetData, FixturePresetId},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeEffectUpdateArgs {
    pub preset_id: FixturePresetId,
    pub new_runtime: KeyframeEffectRuntime,
}

impl FunctionDelegate for KeyframeEffectUpdateArgs {
    fn run(
        &self,
        args: crate::command::parser::nodes::action::ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        let preset = args
            .preset_handler
            .get_preset_mut(self.preset_id)
            .map_err(ActionRunError::PresetHandlerError)?;

        match preset.data_mut() {
            FixturePresetData::KeyframeEffect { runtime } => {
                *runtime = self.new_runtime.clone();
                args.event_list
                    .push(DemexEvent::KeyframeEffectUpdate(self.preset_id));

                Ok(ActionRunResult::Default)
            }
            _ => Err(ActionRunError::PresetHandlerError(
                PresetHandlerError::FeaturePresetNotFound(self.preset_id),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeEffectApplyPresetArgs {
    pub preset_id: FixturePresetId,
    pub effect_preset: KeyframeEffectPreset,
}

impl FunctionDelegate for KeyframeEffectApplyPresetArgs {
    fn run(
        &self,
        args: crate::command::parser::nodes::action::ActionRunArgs,
    ) -> Result<ActionRunResult, ActionRunError> {
        if self.effect_preset.feature_group() != self.preset_id.feature_group {
            return Err(ActionRunError::PresetHandlerError(
                PresetHandlerError::FeatureGroupMismatch(
                    self.effect_preset.feature_group(),
                    self.preset_id.feature_group,
                ),
            ));
        }

        let preset = args
            .preset_handler
            .get_preset_mut(self.preset_id)
            .map_err(ActionRunError::PresetHandlerError)?;

        match preset.data_mut() {
            FixturePresetData::KeyframeEffect { runtime } => {
                runtime
                    .effect_mut()
                    .apply_preset(self.effect_preset.clone());
                args.event_list
                    .push(DemexEvent::KeyframeEffectUpdate(self.preset_id));

                Ok(ActionRunResult::Default)
            }
            _ => Err(ActionRunError::PresetHandlerError(
                PresetHandlerError::FeaturePresetNotFound(self.preset_id),
            )),
        }
    }
}
