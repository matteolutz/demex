use demex_core::effect::speed::{EffectSpeed, EffectSpeedScale, EffectSpeedSyncMode};
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Window,
    prelude::FluentBuilder,
};
use gpui_component::{
    h_flex,
    input::{InputEvent, InputState, NumberInput},
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::ui2::components::effect_speed_editor::select::{
    EffectSpeedScaleSelectItem, EffectSpeedSelectItem, EffectSpeedSyncModeSelectItem,
};

mod select;

pub struct EffectSpeedEditor {
    effect_speed: EffectSpeed,

    select_state: Entity<SelectState<Vec<EffectSpeedSelectItem>>>,

    // BPM
    bpm_input_state: Entity<InputState>,

    // Speedmaster
    speed_master_id_input_state: Entity<InputState>,
    speed_scale_select_state: Entity<SelectState<Vec<EffectSpeedScaleSelectItem>>>,
    speed_sync_mode_select_state: Entity<SelectState<Vec<EffectSpeedSyncModeSelectItem>>>,

    // Speedmaster
    _subscriptions: Vec<Subscription>,
}

impl EffectSpeedEditor {
    pub fn new(effect_speed: EffectSpeed, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let select_state = cx.new(|cx| {
            SelectState::new(
                EffectSpeed::iter().map_into().collect(),
                Some(EffectSpeedSelectItem::index_of(&effect_speed)),
                window,
                cx,
            )
        });

        let bpm_input_state = cx.new(|cx| {
            InputState::new(window, cx).default_value(match effect_speed {
                EffectSpeed::Bpm(bpm) => bpm.to_string(),
                _ => "".into(),
            })
        });

        let speed_master_id_input_state = cx.new(|cx| {
            InputState::new(window, cx).default_value(match effect_speed {
                EffectSpeed::SpeedMaster { id, .. } => id.to_string(),
                _ => "".into(),
            })
        });

        let speed_scale_select_state = cx.new(|cx| {
            SelectState::new(
                EffectSpeedScale::iter().map_into().collect(),
                match effect_speed {
                    EffectSpeed::SpeedMaster { scale, .. } => {
                        Some(EffectSpeedScaleSelectItem::index_of(&scale))
                    }
                    _ => None,
                },
                window,
                cx,
            )
        });

        let speed_sync_mode_select_state = cx.new(|cx| {
            SelectState::new(
                EffectSpeedSyncMode::iter().map_into().collect(),
                match effect_speed {
                    EffectSpeed::SpeedMaster { sync, .. } => {
                        Some(EffectSpeedSyncModeSelectItem::index_of(&sync))
                    }
                    _ => None,
                },
                window,
                cx,
            )
        });

        let _subscriptions = vec![
            cx.subscribe_in(
                &select_state,
                window,
                |this, select, evt: &SelectEvent<Vec<EffectSpeedSelectItem>>, window, cx| match evt
                {
                    SelectEvent::Confirm(_) => {
                        let Some(selected_effect_speed) = select.read(cx).selected_value() else {
                            return;
                        };

                        this.effect_speed = *selected_effect_speed;
                        this.update_inputs_and_selects(window, cx);
                        cx.notify();
                    }
                },
            ),
            cx.subscribe(
                &bpm_input_state,
                |this, bpm_input_state, evt: &InputEvent, cx| match evt {
                    InputEvent::Change => match &mut this.effect_speed {
                        EffectSpeed::Bpm(bpm) => {
                            if let Ok(new_bpm) = bpm_input_state.read(cx).value().parse() {
                                *bpm = new_bpm;
                                cx.notify();
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
            ),
            cx.subscribe(
                &speed_master_id_input_state,
                |this, speed_master_id_input_state, evt: &InputEvent, cx| match evt {
                    InputEvent::Change => match &mut this.effect_speed {
                        EffectSpeed::SpeedMaster { id, .. } => {
                            if let Ok(new_id) = speed_master_id_input_state.read(cx).value().parse()
                            {
                                *id = new_id;
                                cx.notify();
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
            ),
            cx.subscribe(
                &speed_scale_select_state,
                |this, speed_scale, evt: &SelectEvent<Vec<EffectSpeedScaleSelectItem>>, cx| {
                    match evt {
                        SelectEvent::Confirm(_) => {
                            let Some(selected_speed_scale) = speed_scale.read(cx).selected_value()
                            else {
                                return;
                            };

                            match &mut this.effect_speed {
                                EffectSpeed::SpeedMaster { scale, .. } => {
                                    *scale = *selected_speed_scale;
                                    cx.notify();
                                }
                                _ => {}
                            }
                        }
                    }
                },
            ),
            cx.subscribe(
                &speed_sync_mode_select_state,
                |this, speed_sync, evt: &SelectEvent<Vec<EffectSpeedSyncModeSelectItem>>, cx| {
                    match evt {
                        SelectEvent::Confirm(_) => {
                            let Some(selected_speed_sync) = speed_sync.read(cx).selected_value()
                            else {
                                return;
                            };

                            match &mut this.effect_speed {
                                EffectSpeed::SpeedMaster { sync, .. } => {
                                    *sync = *selected_speed_sync;
                                    cx.notify();
                                }
                                _ => {}
                            }
                        }
                    }
                },
            ),
        ];

        Self {
            effect_speed,

            select_state,

            bpm_input_state,

            speed_master_id_input_state,
            speed_scale_select_state,
            speed_sync_mode_select_state,

            _subscriptions,
        }
    }

    fn update_inputs_and_selects(&self, window: &mut Window, cx: &mut Context<Self>) {
        match self.effect_speed {
            EffectSpeed::Bpm(bpm) => self.bpm_input_state.update(cx, |input_state, cx| {
                input_state.set_value(bpm.to_string(), window, cx)
            }),
            EffectSpeed::SpeedMaster { id, scale, sync } => {
                self.speed_master_id_input_state
                    .update(cx, |input_state, cx| {
                        input_state.set_value(id.to_string(), window, cx);
                    });

                self.speed_scale_select_state
                    .update(cx, |select_state, cx| {
                        select_state.set_selected_index(
                            Some(EffectSpeedScaleSelectItem::index_of(&scale)),
                            window,
                            cx,
                        );
                    });

                self.speed_sync_mode_select_state
                    .update(cx, |select_state, cx| {
                        select_state.set_selected_index(
                            Some(EffectSpeedSyncModeSelectItem::index_of(&sync)),
                            window,
                            cx,
                        );
                    });
            }
        }
        cx.notify();
    }

    pub fn set_value(&mut self, value: EffectSpeed, window: &mut Window, cx: &mut Context<Self>) {
        self.effect_speed = value;

        self.select_state.update(cx, |select, cx| {
            select.set_selected_index(Some(EffectSpeedSelectItem::index_of(&value)), window, cx);
        });
        self.update_inputs_and_selects(window, cx);
        cx.notify();
    }

    pub fn value(&self) -> EffectSpeed {
        self.effect_speed
    }
}

impl Render for EffectSpeedEditor {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(Select::new(&self.select_state))
            .when(matches!(self.effect_speed, EffectSpeed::Bpm(_)), |this| {
                this.child(
                    NumberInput::new(&self.bpm_input_state)
                        .suffix("BPM")
                        .w_full(),
                )
            })
            .when(
                matches!(self.effect_speed, EffectSpeed::SpeedMaster { .. }),
                |this| {
                    this.child(NumberInput::new(&self.speed_master_id_input_state).w_full())
                        .child(
                            h_flex()
                                .gap_2()
                                .child(Select::new(&self.speed_scale_select_state))
                                .child(Select::new(&self.speed_sync_mode_select_state)),
                        )
                },
            )
    }
}
