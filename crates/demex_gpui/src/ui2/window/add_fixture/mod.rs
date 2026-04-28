use demex_core::uuid::Uuid;
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, div,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName,
    button::{Button, ButtonVariants},
    input::{Input, InputState, NumberInput},
    label::Label,
    list::{List, ListEvent, ListState},
    v_flex,
};

use crate::{
    engine::state::DemexUiState,
    ui2::{
        window::add_fixture::{
            fixture_mode_list::{FixtureModeListDelegate, FixtureModeTableEntry},
            fixture_type_list::{FixtureTypeListDelegate, FixtureTypeTableEntry},
        },
        wm::edit_window::EditWindowDelegate,
    },
};

mod actions {
    use gpui::{App, KeyBinding};

    pub const CONTEXT: &str = "demex-add-fixture-window";

    gpui::actions!([QuitAddFixture]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new("escape", QuitAddFixture, Some(CONTEXT))]);
    }
}

mod fixture_mode_list;
mod fixture_type_list;

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

#[derive(Debug, Default, Clone)]
pub enum AddFixtureFormState {
    #[default]
    Initial,

    FixtureTypeSelected {
        fixture_type_id: Uuid,
    },

    FixtureTypeAndModeSelected {
        fixture_type_id: Uuid,
        fixture_mode: String,
    },

    FixtureTypeAndModeSubmitted {
        fixture_type_id: Uuid,
        fixture_mode: String,
    },
}

impl AddFixtureFormState {
    pub fn fixture_type_selected(&mut self, fixture_type_id: Uuid, cx: &mut Context<Self>) {
        *self = AddFixtureFormState::FixtureTypeSelected { fixture_type_id };
        cx.notify();
    }

    pub fn fixture_type_deselected(&mut self, cx: &mut Context<Self>) {
        *self = AddFixtureFormState::Initial;
        cx.notify();
    }

    pub fn fixture_mode_selected(&mut self, fixture_mode: String, cx: &mut Context<Self>) {
        if let Some(fixture_type_id) = self.fixture_type_id() {
            *self = AddFixtureFormState::FixtureTypeAndModeSelected {
                fixture_type_id: fixture_type_id,
                fixture_mode,
            };
        } else {
            *self = AddFixtureFormState::Initial;
        }

        cx.notify();
    }

    pub fn fixture_mode_deselected(&mut self, cx: &mut Context<Self>) {
        if let Some(fixture_type_id) = self.fixture_type_id() {
            *self = AddFixtureFormState::FixtureTypeSelected { fixture_type_id };
        } else {
            *self = AddFixtureFormState::Initial;
        }

        cx.notify();
    }

    pub fn submit_fixture_type_and_mode(&mut self, cx: &mut Context<Self>) {
        match self {
            Self::FixtureTypeAndModeSelected {
                fixture_type_id,
                fixture_mode,
            } => {
                *self = AddFixtureFormState::FixtureTypeAndModeSubmitted {
                    fixture_type_id: *fixture_type_id,
                    fixture_mode: fixture_mode.clone(),
                };
                cx.notify();
            }
            _ => {}
        }
    }

    pub fn back_to_fixture_type_and_mode(&mut self, cx: &mut Context<Self>) {
        match self {
            AddFixtureFormState::FixtureTypeAndModeSubmitted {
                fixture_type_id,
                fixture_mode,
            } => {
                *self = AddFixtureFormState::FixtureTypeAndModeSelected {
                    fixture_type_id: *fixture_type_id,
                    fixture_mode: fixture_mode.clone(),
                };
                cx.notify();
            }
            _ => {}
        }
    }

    pub fn fixture_type_id(&self) -> Option<Uuid> {
        match self {
            AddFixtureFormState::FixtureTypeSelected { fixture_type_id }
            | AddFixtureFormState::FixtureTypeAndModeSelected {
                fixture_type_id, ..
            } => Some(*fixture_type_id),
            _ => None,
        }
    }
}

pub struct AddFixtureWindow {
    form_state: Entity<AddFixtureFormState>,

    fixture_type_list_state: Entity<ListState<FixtureTypeListDelegate>>,
    fixture_mode_list_state: Entity<ListState<FixtureModeListDelegate>>,

    name_input_state: Entity<InputState>,
    quantity_input_state: Entity<InputState>,
    starting_fixture_path_input_state: Entity<InputState>,
    starting_patch_input_state: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl AddFixtureWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let form_state = cx.new(|_| AddFixtureFormState::default());

        let fixture_type_list_state = cx.new(|cx| {
            ListState::new(
                FixtureTypeListDelegate::new(
                    DemexUiState::patch(cx)
                        .read(cx)
                        .fixture_types()
                        .iter()
                        .map(|ft| FixtureTypeTableEntry {
                            fixture_type_id: ft.fixture_type_id,
                            name: ft.long_name.clone().into(),
                        }),
                ),
                window,
                cx,
            )
        });

        let fixture_mode_list_state =
            cx.new(|cx| ListState::new(FixtureModeListDelegate::new(vec![]), window, cx));

        let name_input_state = cx.new(|cx| InputState::new(window, cx));
        let quantity_input_state = cx.new(|cx| InputState::new(window, cx).default_value("1"));
        let starting_fixture_path_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("1"));
        let starting_patch_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("1.001"));

        let _subscriptions = vec![
            cx.subscribe(
                &fixture_type_list_state,
                |this, list_state, evt, cx| match evt {
                    ListEvent::Cancel => {
                        this.form_state
                            .update(cx, |form_state, cx| form_state.fixture_type_deselected(cx));
                        cx.notify();
                    }
                    ListEvent::Select(item) | ListEvent::Confirm(item) => {
                        let idx = item.row;
                        let fixture_type_id =
                            list_state.read(cx).delegate().items[idx].fixture_type_id;

                        this.form_state.update(cx, |form_state, cx| {
                            form_state.fixture_type_selected(fixture_type_id, cx)
                        });
                        cx.notify();
                    }
                },
            ),
            cx.subscribe(
                &fixture_mode_list_state,
                |this, list_state, evt, cx| match evt {
                    ListEvent::Cancel => {
                        this.form_state.update(cx, |form_state, cx| {
                            form_state.fixture_mode_deselected(cx);
                        });
                        cx.notify();
                    }
                    ListEvent::Select(item) | ListEvent::Confirm(item) => {
                        let idx = item.row;
                        let fixture_mode =
                            list_state.read(cx).delegate().items[idx].name.to_string();

                        this.form_state.update(cx, |form_state, cx| {
                            form_state.fixture_mode_selected(fixture_mode, cx)
                        });
                        cx.notify();
                    }
                },
            ),
            cx.observe(&form_state, |this, form_state, cx| {
                this.set_edited(true, cx);

                match form_state.read(cx) {
                    AddFixtureFormState::FixtureTypeSelected { fixture_type_id } => {
                        if let Some(fixture_type) = DemexUiState::patch(cx)
                            .read(cx)
                            .fixture_type(*fixture_type_id)
                        {
                            let modes = fixture_type
                                .dmx_modes
                                .iter()
                                .map(|mode| FixtureModeTableEntry {
                                    name: mode
                                        .name
                                        .as_ref()
                                        .map(|name| name.to_string())
                                        .unwrap_or_default()
                                        .into(),
                                })
                                .collect::<Vec<_>>();

                            this.fixture_mode_list_state.update(cx, |mode_list, cx| {
                                mode_list.delegate_mut().update_items(modes, cx);
                            });
                            cx.notify();
                        }
                    }
                    _ => {}
                }
            }),
        ];

        Self {
            form_state,
            fixture_type_list_state,
            fixture_mode_list_state,

            name_input_state,
            quantity_input_state,
            starting_fixture_path_input_state,
            starting_patch_input_state,

            _subscriptions,
        }
    }
}

impl EditWindowDelegate for AddFixtureWindow {
    fn window_title(&self, _window: &mut Window, _cx: &App) -> impl Into<gpui::SharedString> {
        "Add Fixture"
    }

    fn handle_save(&self, _window: &mut Window, _cx: &mut App) {}

    fn handle_discard(&self, _window: &mut Window, _cx: &mut App) {}

    fn should_have_save_button(_cx: &App) -> bool
    where
        Self: Sized,
    {
        false
    }
}

impl AddFixtureWindow {
    fn render_gdtf_lists(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<AddFixtureWindow>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .p_2()
            .gap_2()
            .size_full()
            .child(
                div()
                    .size_full()
                    .gap_2()
                    .grid()
                    .grid_rows(1)
                    .grid_cols(2)
                    .child(
                        List::new(&self.fixture_type_list_state)
                            .border_color(cx.theme().border)
                            .border_1()
                            .size_full()
                            .h_full(),
                    )
                    .child(
                        List::new(&self.fixture_mode_list_state)
                            .border_color(cx.theme().border)
                            .border_1()
                            .size_full()
                            .h_full(),
                    ),
            )
            .child(
                div().p_4().flex().justify_end().child(
                    Button::new("next")
                        .label("Next")
                        .icon(IconName::ChevronRight)
                        .disabled(!matches!(
                            self.form_state.read(cx),
                            AddFixtureFormState::FixtureTypeAndModeSelected { .. }
                        ))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.form_state.update(cx, |form_state, cx| {
                                form_state.submit_fixture_type_and_mode(cx)
                            });
                            cx.notify();
                        })),
                ),
            )
    }

    fn render_patch_inputs(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .p_2()
            .gap_2()
            .size_full()
            .child(
                v_flex()
                    .p_4()
                    .gap_4()
                    .size_full()
                    .child(
                        v_flex()
                            .child(
                                Label::new("Name")
                            )
                            .child(div().text_color(cx.theme().muted_foreground).text_sm().child("Use %i for the fixture index (starting at 1), %fp for the fixture path and %pt for the patch"))
                            .child(Input::new(&self.name_input_state)),
                    )
                    .child(
                        v_flex()
                            .child(Label::new("Quantity"))
                            .child(NumberInput::new(&self.quantity_input_state)),
                    )
                    .child(
                        v_flex()
                            .child(Label::new("Starting Fixture Path"))
                            .child(NumberInput::new(&self.starting_fixture_path_input_state)),
                    )
                    .child(
                        v_flex()
                            .child(Label::new("Starting Patch"))
                            .child(Input::new(&self.starting_patch_input_state)),
                    ),
            )
            .child(
                div().p_4().flex().justify_between().child(
                    Button::new("prev")
                        .icon(IconName::ChevronLeft)
                        .label("Edit Type and Mode")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.form_state.update(cx, |form_state, cx| {
                                form_state.back_to_fixture_type_and_mode(cx)
                            });
                            cx.notify();
                        }))
                )
                .child(
                    Button::new("patch")
                        .primary()
                        .disabled(true)
                        .label("Patch!")
                )
            )
    }
}

impl Render for AddFixtureWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        match self.form_state.read(cx) {
            AddFixtureFormState::FixtureTypeAndModeSubmitted { .. } => {
                self.render_patch_inputs(window, cx).into_any_element()
            }
            _ => self.render_gdtf_lists(window, cx).into_any_element(),
        }
    }
}
