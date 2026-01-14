use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    command::{
        lexer::token::Token,
        parser::nodes::{
            action::{
                Action, ActionIssuer, ValueOrRange,
                functions::{
                    go_function::ExecutorGoArgs,
                    set_function::{SelectionOrSelector, SetFixturePresetArgs},
                    speedmaster_functions::SpeedMasterTapArgs,
                    start_function::ExecutorStartArgs,
                    stomp_function::ExecutorStompArgs,
                    stop_function::ExecutorStopArgs,
                },
                queue::ActionQueue,
            },
            fixture_selector::{FixtureSelector, FixtureSelectorContext, FixtureSelectorError},
        },
    },
    event::{DemexEvent, list::DemexEventList},
    input::{
        DemexInputDeviceUpdateArgs, control::DemexInputDeviceControlTrait,
        error::DemexInputDeviceError, event::DemexInputDeviceButtonUpdate,
    },
    patch::Patch,
    presets::{PresetHandler, preset::FixturePresetId},
    selection::FixtureSelection,
    state::fixture_state_handler::FixtureStateHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum DemexInputButton {
    ExecutorGo(u32),
    ExecutorStop(u32),
    ExecutorFlash {
        id: u32,
        stomp: bool,
    },

    SelectivePreset {
        selection: Option<FixtureSelection>,
        preset_id: FixturePresetId,
    },

    #[warn(deprecated)]
    FixtureSelector {
        fixture_selector: FixtureSelector,
    },

    SpeedMasterTap {
        speed_master_id: u32,
    },

    Macro {
        action: Action,
    },

    #[default]
    Unused,
}

impl DemexInputButton {
    pub fn handle_press(
        &self,
        action_queue: &mut ActionQueue,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            &Self::ExecutorFlash {
                id: executor_id,
                stomp,
            } => {
                action_queue.enqueue_now(
                    Action::ExecutorStart(ExecutorStartArgs { executor_id }),
                    ActionIssuer::InputDevice,
                );

                if stomp {
                    action_queue.enqueue_now(
                        Action::ExecutorStomp(ExecutorStompArgs {
                            executor_id,
                            stomped: true,
                        }),
                        ActionIssuer::InputDevice,
                    );
                }
            }
            &Self::ExecutorGo(executor_id) => {
                action_queue.enqueue_now(
                    Action::ExecutorGo(ExecutorGoArgs { executor_id }),
                    ActionIssuer::InputDevice,
                );
            }
            &Self::ExecutorStop(executor_id) => {
                action_queue.enqueue_now(
                    Action::ExecutorStop(ExecutorStopArgs { executor_id }),
                    ActionIssuer::InputDevice,
                );
            }
            Self::SelectivePreset {
                selection,
                preset_id,
            } => {
                action_queue.enqueue_now(
                    Action::SetFixturePreset(SetFixturePresetArgs {
                        selection_or_selector: selection
                            .clone()
                            .map(|sel| SelectionOrSelector::Selection(sel))
                            .unwrap_or(SelectionOrSelector::Current),
                        preset_id: (*preset_id).into(),
                    }),
                    ActionIssuer::InputDevice,
                );
            }
            Self::Macro { action } => {
                action_queue.enqueue_now(action.clone(), ActionIssuer::Macro);
            }
            Self::FixtureSelector { fixture_selector } => {
                action_queue.enqueue_now(
                    Action::FixtureSelector(fixture_selector.clone()),
                    ActionIssuer::InputDevice,
                );
            }
            &Self::SpeedMasterTap { speed_master_id } => {
                action_queue.enqueue_now(
                    Action::SpeedMasterTap(SpeedMasterTapArgs {
                        speedmaster_id: speed_master_id,
                    }),
                    ActionIssuer::InputDevice,
                );
            }
            Self::Unused => {}
        };

        Ok(())
    }

    pub fn handle_release(
        &self,
        action_queue: &mut ActionQueue,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            &Self::ExecutorFlash {
                id: executor_id,
                stomp,
            } => {
                action_queue.enqueue_now(
                    Action::ExecutorStop(ExecutorStopArgs { executor_id }),
                    ActionIssuer::InputDevice,
                );

                if stomp {
                    action_queue.enqueue_now(
                        Action::ExecutorStomp(ExecutorStompArgs {
                            executor_id,
                            stomped: false,
                        }),
                        ActionIssuer::InputDevice,
                    );
                }
            }
            _ => {}
        };

        Ok(())
    }
}

impl DemexInputDeviceControlTrait<DemexInputDeviceButtonUpdate> for DemexInputButton {
    fn initial_state(
        &self,
        args: DemexInputDeviceUpdateArgs,
    ) -> Result<DemexInputDeviceButtonUpdate, DemexInputDeviceError> {
        todo!()
        /*
        match self {
            Self::ExecutorFlash { id, .. } | Self::ExecutorGo(id) | Self::ExecutorStop(id) => {
                let executor = args
                    .updatable_handler
                    .executor(*id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                if executor.is_active() {
                    Ok(DemexInputDeviceButtonUpdate::ButtonActive)
                } else {
                    Ok(DemexInputDeviceButtonUpdate::ButtonInactive)
                }
            }
            _ => Ok(DemexInputDeviceButtonUpdate::default()),
        }
        */
    }

    fn should_update(
        &self,
        _args: DemexInputDeviceUpdateArgs,
        event: &DemexEvent,
    ) -> Result<Option<DemexInputDeviceButtonUpdate>, DemexInputDeviceError> {
        let update = match self {
            Self::ExecutorFlash { id, .. } | Self::ExecutorGo(id) | Self::ExecutorStop(id) => {
                match event {
                    DemexEvent::ExecutorGo(event_id) if event_id == id => {
                        Some(DemexInputDeviceButtonUpdate::ButtonActive)
                    }
                    DemexEvent::ExecutorStop(event_id) if event_id == id => {
                        Some(DemexInputDeviceButtonUpdate::ButtonInactive)
                    }
                    _ => None,
                }
            }
            Self::FixtureSelector { .. } => None,
            Self::SelectivePreset { .. } => None,
            Self::Macro { .. } => None,
            Self::SpeedMasterTap { .. } => None,
            Self::Unused => None,
        };

        Ok(update)
    }
}
