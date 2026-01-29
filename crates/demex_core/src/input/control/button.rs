use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::{
        action::{
            Action, ActionIssuer,
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
        control::{DemexInputDeviceControlAssignmentDelegate, DemexInputDeviceControlDelegate},
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
    ) -> Result<
        (
            Self::Control,
            Option<<Self::Control as DemexInputDeviceControlDelegate>::Update>,
        ),
        DemexInputDeviceError,
    > {
        match self {
            DemexInputButtonAssignment::ExecutorGo {
                executor_id,
                is_running,
            } => Ok((
                DemexInputButton::ExecutorGo(executor_id),
                Some(if is_running {
                    DemexInputDeviceButtonUpdate::ButtonActive
                } else {
                    DemexInputDeviceButtonUpdate::ButtonInactive
                }),
            )),
            DemexInputButtonAssignment::ExecutorStop {
                executor_id,
                is_running,
            } => Ok((
                DemexInputButton::ExecutorStop(executor_id),
                Some(if is_running {
                    DemexInputDeviceButtonUpdate::ButtonActive
                } else {
                    DemexInputDeviceButtonUpdate::ButtonInactive
                }),
            )),
            DemexInputButtonAssignment::ExecutorFlash {
                executor_id,
                stomp,
                is_running,
            } => Ok((
                DemexInputButton::ExecutorFlash {
                    id: executor_id,
                    stomp,
                },
                Some(if is_running {
                    DemexInputDeviceButtonUpdate::ButtonActive
                } else {
                    DemexInputDeviceButtonUpdate::ButtonInactive
                }),
            )),
            DemexInputButtonAssignment::SelectivePreset {
                selection,
                preset_id,
            } => Ok((
                DemexInputButton::SelectivePreset {
                    selection,
                    preset_id,
                },
                None,
            )),
            DemexInputButtonAssignment::FixtureSelector { fixture_selector } => {
                Ok((DemexInputButton::FixtureSelector { fixture_selector }, None))
            }
            DemexInputButtonAssignment::SpeedMasterTap { speed_master_id } => {
                Ok((DemexInputButton::SpeedMasterTap { speed_master_id }, None))
            }
            DemexInputButtonAssignment::Macro { action } => {
                Ok((DemexInputButton::Macro { action }, None))
            }
            DemexInputButtonAssignment::Unused => Ok((DemexInputButton::Unused, None)),
        }
    }
}
