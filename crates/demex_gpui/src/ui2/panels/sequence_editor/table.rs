use std::{
    collections::HashMap,
    rc::Rc,
    time::{self, Duration},
};

use demex_core::{
    command::parser::nodes::{
        action::{Action, functions::set_function::ObjectSetPropertyArgs},
        object::Object,
    },
    sequence::{
        cue::{CueFadingFunction, CueIdx, CueProperty},
        frontend::FrontendCue,
    },
};
use gpui::{
    App, Context, DefiniteLength, InteractiveElement, IntoElement, ParentElement, Styled, Task, div,
};
use gpui_component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    menu::{DropdownMenu, PopupMenuItem},
    table::{Column, TableDelegate, TableState},
};
use strum::IntoEnumIterator;

use crate::{
    engine::DemexEngineHandler,
    ui2::{
        config::AppConfigExt,
        window::{
            edit_cue_trigger::EditCueTriggerWindow,
            set_property::{SetPropertyWindow, SetPropertyWindowPropertyType},
        },
        wm::{WindowManager, edit_window::WindowManagerExtension},
    },
};

pub struct SequenceEditorTable {
    data: Option<(u32, Vec<FrontendCue>)>,
    active_cues: HashMap<CueIdx, time::Instant>,

    next_render: Option<Task<()>>,

    columns: Vec<Column>,
}

impl SequenceEditorTable {
    pub fn new(data: Option<(u32, Vec<FrontendCue>)>) -> Self {
        Self {
            data,
            active_cues: HashMap::new(),
            columns: vec![
                Column::new("id", "Id").width(60.0),
                Column::new("name", "Name").width(150.0),
                Column::new("in-fade", "In Fade").width(60.0),
                Column::new("in-delay", "In Delay").width(60.0),
                Column::new("snap-percent", "Snap %").width(60.0),
                Column::new("block", "Block").width(60.0),
                Column::new("mib", "Move In Black").width(60.0),
                Column::new("trigger", "Trigger").width(80.0),
                Column::new("fading", "Fading").width(80.0),
            ],
            next_render: None,
        }
    }

    pub fn update_data(&mut self, data: Option<(u32, Vec<FrontendCue>)>) {
        self.data = data;
        self.active_cues.clear();
    }

    pub fn update_active_cues(&mut self, active_cues: Option<Vec<(CueIdx, time::Instant)>>) {
        if let Some(active_cues) = active_cues {
            self.active_cues = active_cues.into_iter().collect();
        } else {
            self.active_cues.clear();
        }
    }

    pub fn cue_activated(&mut self, cue_idx: CueIdx, at: time::Instant) {
        self.active_cues.insert(cue_idx, at);
    }

    pub fn cue_deactivated(&mut self, cue_idx: &CueIdx) {
        self.active_cues.remove(&cue_idx);
    }

    pub fn executor_stop(&mut self) {
        self.active_cues.clear();
    }

    fn set_property(
        (seq_id, cue_idx): (u32, CueIdx),
        key: CueProperty,
        value: impl ToString,
        cx: &mut App,
    ) {
        DemexEngineHandler::engine(cx).exec_ui(Action::ObjectSetProperty(ObjectSetPropertyArgs {
            object: Object::cue(seq_id, cue_idx),
            key: key.to_string(),
            value: value.to_string(),
        }));
    }
}

impl TableDelegate for SequenceEditorTable {
    fn loading(&self, _cx: &gpui::App) -> bool {
        self.data.is_none()
    }

    fn columns_count(&self, _cx: &gpui::App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &gpui::App) -> usize {
        self.data.as_ref().map(|data| data.1.len()).unwrap_or(0)
    }

    fn column(&self, col_ix: usize, _cx: &gpui::App) -> gpui_component::table::Column {
        self.columns[col_ix].clone()
    }

    fn render_tr(
        &mut self,
        row_ix: usize,
        _window: &mut gpui::Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> gpui::Stateful<gpui::Div> {
        div().id(("row", row_ix)).h_10().items_center()
    }

    fn render_header(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<TableState<Self>>,
    ) -> gpui::Stateful<gpui::Div> {
        if !self.active_cues.is_empty() {
            let _ = self.next_render.take();

            self.next_render = Some(cx.spawn(async |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_secs_f64(1.0 / 60.0))
                    .await;

                let _ = this.update(cx, |this, cx| {
                    this.refresh(cx);
                    cx.notify();
                });
            }));
        }

        div().id("header")
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut gpui::Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl gpui::IntoElement {
        let Some((sequence_id, data)) = self.data.as_ref() else {
            return div().into_any_element();
        };

        let column = &self.columns[col_ix];
        let cue = &data[row_ix];

        let activated_at = self.active_cues.get(&cue.cue_idx);
        let current_fade = activated_at
            .map(|activated| (activated.elapsed().as_secs_f32() - cue.in_delay) / cue.in_fade)
            .map(|fade| fade.clamp(0.0, 1.0))
            .unwrap_or(0.0);

        let sequence_id = *sequence_id;
        let cue_idx = cue.cue_idx;

        match column.key.as_ref() {
            "id" => div()
                .flex()
                .items_center()
                .size_full()
                .child(cue_idx.to_string())
                .into_any_element(),
            "name" => div()
                .size_full()
                .relative()
                .flex()
                .items_center()
                .child(
                    div()
                        .bg(cx.theme().blue)
                        .absolute()
                        .top_0()
                        .left_0()
                        .h_full()
                        .w(DefiniteLength::Fraction(current_fade)),
                )
                .child(
                    Button::new("edit-name")
                        .on_click(move |_, _, cx| {
                            WindowManager::open_edit_window::<SetPropertyWindow>(
                                cx,
                                move |window, cx| {
                                    SetPropertyWindow::new(
                                        Object::SequenceCue(sequence_id, cue_idx),
                                        CueProperty::Name,
                                        SetPropertyWindowPropertyType::String,
                                        window,
                                        cx,
                                    )
                                },
                            );
                        })
                        .text()
                        .label(cue.name.clone()),
                )
                .into_any_element(),
            "in-fade" => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Button::new("edit-in-fade")
                        .on_click(move |_, _, cx| {
                            WindowManager::open_edit_window::<SetPropertyWindow>(
                                cx,
                                move |window, cx| {
                                    SetPropertyWindow::new(
                                        Object::SequenceCue(sequence_id, cue_idx),
                                        CueProperty::InFade,
                                        SetPropertyWindowPropertyType::relative_positive_seconds(),
                                        window,
                                        cx,
                                    )
                                },
                            );
                        })
                        .text()
                        .label(format!("{:.2}s", cue.in_fade)),
                )
                .into_any_element(),
            "in-delay" => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Button::new("edit-in-delay")
                        .on_click(move |_, _, cx| {
                            WindowManager::open_edit_window::<SetPropertyWindow>(
                                cx,
                                move |window, cx| {
                                    SetPropertyWindow::new(
                                        Object::SequenceCue(sequence_id, cue_idx),
                                        CueProperty::InDelay,
                                        SetPropertyWindowPropertyType::relative_positive_seconds(),
                                        window,
                                        cx,
                                    )
                                },
                            );
                        })
                        .text()
                        .label(format!("{:.2}s", cue.in_delay)),
                )
                .into_any_element(),
            "snap-percent" => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Button::new("edit-snap-percent")
                        .on_click(move |_, _, cx| {
                            WindowManager::open_edit_window::<SetPropertyWindow>(
                                cx,
                                move |window, cx| {
                                    SetPropertyWindow::new(
                                        Object::SequenceCue(sequence_id, cue_idx),
                                        CueProperty::SnapPercent,
                                        SetPropertyWindowPropertyType::Percentage,
                                        window,
                                        cx,
                                    )
                                },
                            );
                        })
                        .text()
                        .label(format!("{}%", cue.snap_percent * 100.0)),
                )
                .into_any_element(),
            "block" => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Checkbox::new("edit-block")
                        .with_size(cx.ui_config().ui_size())
                        .checked(cue.block)
                        .on_click(move |value, _, cx| {
                            Self::set_property(
                                (sequence_id, cue_idx),
                                CueProperty::Block,
                                value,
                                cx,
                            );
                        }),
                )
                .into_any_element(),
            "mib" => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Checkbox::new("edit-mib")
                        .with_size(cx.ui_config().ui_size())
                        .checked(cue.move_in_black)
                        .on_click(move |value, _, cx| {
                            Self::set_property(
                                (sequence_id, cue_idx),
                                CueProperty::MoveInBlack,
                                value,
                                cx,
                            );
                        }),
                )
                .into_any_element(),
            "trigger" => Button::new("edit-trigger")
                .ghost()
                .label(cue.trigger.to_pretty_string())
                .on_click(move |_, _, cx| {
                    WindowManager::open_edit_window::<EditCueTriggerWindow>(
                        cx,
                        move |window, cx| {
                            EditCueTriggerWindow::new(sequence_id, cue_idx, window, cx)
                        },
                    );
                })
                .into_any_element(),
            "fading" => Button::new("edit-fading")
                .ghost()
                .label(format!("{}", cue.fading_function))
                .dropdown_menu({
                    move |mut menu, _, _| {
                        for ff in CueFadingFunction::iter() {
                            menu = menu.item(PopupMenuItem::Item {
                                icon: None,
                                label: ff.to_string().into(),
                                disabled: false,
                                checked: false,
                                action: None,
                                is_link: false,
                                handler: Some(Rc::new(move |_, _, cx| {
                                    Self::set_property(
                                        (sequence_id, cue_idx),
                                        CueProperty::FadingFunction,
                                        ff,
                                        cx,
                                    );
                                })),
                            })
                        }
                        menu
                    }
                })
                .into_any_element(),
            _ => unreachable!(),
        }
    }
}
