use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        patch::Patch,
        presets::{preset::FixturePresetId, PresetHandler},
        timing::TimingHandler,
        updatables::{error::UpdatableHandlerError, UpdatableHandler},
    },
    input::{button::DemexInputButton, error::DemexInputDeviceError, fader::DemexInputFader},
    lexer::token::Token,
    parser::nodes::{
        action::{error::ActionRunError, result::ActionRunResult, Action, ValueOrRange},
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
};

use super::FunctionArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignButtonArgsMode {
    ExecutorStartAndNext(u32),
    ExecutorStop(u32),
    ExecutorFlash {
        id: u32,
        stomp: bool,
    },
    FaderGo(u32),
    FixtureSelector(FixtureSelector),
    SelectivePreset {
        preset_id_range: ValueOrRange<FixturePresetId>,
        fixture_selector: Option<FixtureSelector>,
    },
    Macro(Box<Action>),
    Tokens(Vec<Token>),
}

impl AssignButtonArgsMode {
    pub fn check_existing(
        &self,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
    ) -> Result<(), ActionRunError> {
        match self {
            Self::ExecutorStop(id)
            | Self::ExecutorFlash { id, .. }
            | Self::ExecutorStartAndNext(id) => {
                updatable_handler
                    .executor(*id)
                    .ok_or(ActionRunError::UpdatableHandlerError(
                        UpdatableHandlerError::UpdatableNotFound(*id),
                    ))?;
            }
            Self::FaderGo(id) => {
                updatable_handler
                    .fader(*id)
                    .map_err(ActionRunError::UpdatableHandlerError)?;
            }
            Self::SelectivePreset {
                preset_id_range: preset_id,
                ..
            } => {
                let (preset_id_from, preset_id_to) = (*preset_id).into();

                preset_handler
                    .get_preset_range(preset_id_from, preset_id_to)
                    .map_err(ActionRunError::PresetHandlerError)?;
            }
            Self::Tokens(_) | Self::FixtureSelector(_) | Self::Macro(_) => {}
        };

        Ok(())
    }

    pub fn to_buttons(
        &self,
        preset_handler: &PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<Vec<DemexInputButton>, ActionRunError> {
        match &self {
            AssignButtonArgsMode::ExecutorStartAndNext(executor_id) => {
                Ok(vec![DemexInputButton::ExecutorStartAndNext(*executor_id)])
            }
            AssignButtonArgsMode::ExecutorStop(executor_id) => {
                Ok(vec![DemexInputButton::ExecutorStop(*executor_id)])
            }
            AssignButtonArgsMode::ExecutorFlash { id, stomp } => {
                Ok(vec![DemexInputButton::ExecutorFlash {
                    id: *id,
                    stomp: *stomp,
                }])
            }
            AssignButtonArgsMode::FaderGo(fader_id) => {
                Ok(vec![DemexInputButton::FaderGo(*fader_id)])
            }
            AssignButtonArgsMode::FixtureSelector(fixture_selector) => {
                Ok(vec![DemexInputButton::FixtureSelector {
                    fixture_selector: fixture_selector.clone(),
                }])
            }
            AssignButtonArgsMode::SelectivePreset {
                preset_id_range,
                fixture_selector,
            } => {
                let selection = if let Some(fs) = fixture_selector {
                    Some(
                        fs.get_selection(preset_handler, fixture_selector_context)
                            .map_err(ActionRunError::FixtureSelectorError)?,
                    )
                } else {
                    None
                };

                Ok(preset_id_range
                    .try_into_id_list()
                    .map_err(ActionRunError::PresetHandlerError)?
                    .into_iter()
                    .map(|preset_id| DemexInputButton::SelectivePreset {
                        preset_id,
                        selection: selection.clone(),
                    })
                    .collect::<Vec<_>>())
            }
            AssignButtonArgsMode::Macro(action) => Ok(vec![DemexInputButton::Macro {
                action: *action.clone(),
            }]),
            AssignButtonArgsMode::Tokens(tokens) => Ok(vec![DemexInputButton::TokenInsert {
                tokens: tokens.clone(),
            }]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignButtonArgs {
    pub mode: AssignButtonArgsMode,
    pub device_idx: usize,
    pub button_id: u32,
}

impl FunctionArgs for AssignButtonArgs {
    fn run(
        &self,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<
        crate::parser::nodes::action::result::ActionRunResult,
        crate::parser::nodes::action::error::ActionRunError,
    > {
        self.mode
            .check_existing(preset_handler, updatable_handler)?;

        let device = input_device_handler
            .device_mut(self.device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.buttons().get(&self.button_id).is_some() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::ButtonAlreadyAssigned(self.button_id),
            ));
        }

        for (idx, button) in self
            .mode
            .to_buttons(preset_handler, fixture_selector_context)?
            .into_iter()
            .enumerate()
        {
            device
                .config
                .buttons_mut()
                .insert(self.button_id + idx as u32, button);
        }

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignFaderArgs {
    pub fader_id: u32,
    pub device_idx: usize,
    pub input_fader_id: u32,
}

impl FunctionArgs for AssignFaderArgs {
    fn run(
        &self,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        _preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        let _ = updatable_handler
            .fader(self.fader_id)
            .map_err(ActionRunError::UpdatableHandlerError)?;

        let device = input_device_handler
            .device_mut(self.device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.faders().get(&self.input_fader_id).is_some() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::FaderAlreadyAssigned(self.input_fader_id),
            ));
        }

        device
            .config
            .faders_mut()
            .insert(self.input_fader_id, DemexInputFader::new(self.fader_id));

        Ok(ActionRunResult::new())
    }
}
