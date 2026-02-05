use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::{
        action::{
            Action, ActionIssuer, ActionRunArgs,
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
        fixture_selector::FixtureSelector,
    },
    event::DemexEvent,
    input::{
        DemexInputDeviceUpdateArgs,
        control::{
            DemexInputControlAssignmentResult, DemexInputDeviceControlAssignmentDelegate,
            DemexInputDeviceControlDelegate,
        },
        error::DemexInputDeviceError,
        event::DemexInputDeviceButtonUpdate,
    },
    presets::preset::FixturePresetId,
    selection::FixtureSelection,
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

impl DemexInputDeviceControlDelegate for DemexInputButton {
    type Update = DemexInputDeviceButtonUpdate;

    fn map_event(
        &self,
        _args: DemexInputDeviceUpdateArgs,
        event: &DemexEvent,
    ) -> Result<Option<Self::Update>, DemexInputDeviceError> {
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

#[derive(Debug, Default, Clone)]
pub enum DemexInputButtonAssignment {
    ExecutorGo {
        executor_id: u32,
        is_running: bool,
    },
    ExecutorStop {
        executor_id: u32,
        is_running: bool,
    },
    ExecutorFlash {
        executor_id: u32,
        stomp: bool,

        is_running: bool,
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

impl DemexInputDeviceControlAssignmentDelegate for DemexInputButtonAssignment {
    type Control = DemexInputButton;

    fn assign(
        self,
    ) -> Result<DemexInputControlAssignmentResult<Self::Control>, DemexInputDeviceError> {
        match self {
            DemexInputButtonAssignment::ExecutorGo {
                executor_id,
                is_running,
            } => Ok(DemexInputControlAssignmentResult {
                control: DemexInputButton::ExecutorGo(executor_id),
                init_event: Some(if is_running {
                    DemexInputDeviceButtonUpdate::ButtonActive
                } else {
                    DemexInputDeviceButtonUpdate::ButtonInactive
                }),
            }),
            DemexInputButtonAssignment::ExecutorStop {
                executor_id,
                is_running,
            } => Ok(DemexInputControlAssignmentResult {
                control: DemexInputButton::ExecutorStop(executor_id),
                init_event: Some(if is_running {
                    DemexInputDeviceButtonUpdate::ButtonActive
                } else {
                    DemexInputDeviceButtonUpdate::ButtonInactive
                }),
            }),
            DemexInputButtonAssignment::ExecutorFlash {
                executor_id,
                stomp,
                is_running,
            } => Ok(DemexInputControlAssignmentResult {
                control: DemexInputButton::ExecutorFlash {
                    id: executor_id,
                    stomp,
                },
                init_event: Some(if is_running {
                    DemexInputDeviceButtonUpdate::ButtonActive
                } else {
                    DemexInputDeviceButtonUpdate::ButtonInactive
                }),
            }),
            DemexInputButtonAssignment::SelectivePreset {
                selection,
                preset_id,
            } => Ok(DemexInputControlAssignmentResult {
                control: DemexInputButton::SelectivePreset {
                    selection,
                    preset_id,
                },
                init_event: None,
            }),
            DemexInputButtonAssignment::FixtureSelector { fixture_selector } => {
                Ok(DemexInputControlAssignmentResult {
                    control: DemexInputButton::FixtureSelector { fixture_selector },
                    init_event: None,
                })
            }
            DemexInputButtonAssignment::SpeedMasterTap { speed_master_id } => {
                Ok(DemexInputControlAssignmentResult {
                    control: DemexInputButton::SpeedMasterTap { speed_master_id },
                    init_event: None,
                })
            }
            DemexInputButtonAssignment::Macro { action } => Ok(DemexInputControlAssignmentResult {
                control: DemexInputButton::Macro { action },
                init_event: None,
            }),
            DemexInputButtonAssignment::Unused => Ok(DemexInputControlAssignmentResult {
                control: DemexInputButton::Unused,
                init_event: None,
            }),
        }
    }

    fn from_control(
        control: Self::Control,
        args: &ActionRunArgs,
    ) -> Result<Self, DemexInputDeviceError> {
        let assignment = match control {
            DemexInputButton::ExecutorGo(executor_id) => {
                let is_running = args
                    .updatable_handler
                    .executor(executor_id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?
                    .is_active();

                DemexInputButtonAssignment::ExecutorGo {
                    executor_id,
                    is_running,
                }
            }
            DemexInputButton::ExecutorStop(executor_id) => {
                let is_running = args
                    .updatable_handler
                    .executor(executor_id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?
                    .is_active();

                DemexInputButtonAssignment::ExecutorStop {
                    executor_id,
                    is_running,
                }
            }
            DemexInputButton::ExecutorFlash { id, stomp } => {
                let is_running = args
                    .updatable_handler
                    .executor(id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?
                    .is_active();

                DemexInputButtonAssignment::ExecutorFlash {
                    executor_id: id,
                    stomp: stomp,
                    is_running,
                }
            }
            DemexInputButton::FixtureSelector { fixture_selector } => {
                DemexInputButtonAssignment::FixtureSelector {
                    fixture_selector: fixture_selector.clone(),
                }
            }
            DemexInputButton::SelectivePreset {
                selection,
                preset_id,
            } => DemexInputButtonAssignment::SelectivePreset {
                preset_id,
                selection: selection.clone(),
            },
            DemexInputButton::Macro { action } => DemexInputButtonAssignment::Macro {
                action: action.clone(),
            },
            DemexInputButton::SpeedMasterTap { speed_master_id } => {
                // make sure the speed master exists
                args.timing_handler
                    .get_speed_master_value(speed_master_id)
                    .map_err(DemexInputDeviceError::TimingHandlerError)?;

                DemexInputButtonAssignment::SpeedMasterTap { speed_master_id }
            }
            DemexInputButton::Unused => Err(DemexInputDeviceError::UnusedButton)?,
        };

        Ok(assignment)
    }
}
