use demex_core::pool::PoolType;
use gpui::{
    App, AppContext, Context, Entity, ParentElement, Render, SharedString, Styled, Subscription,
    Window, WindowBounds, size,
};
use gpui_component::{
    button::Button,
    input::{InputEvent, InputState, NumberInput},
    select::{Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};
use itertools::Itertools;

use crate::ui2::{
    panels::{multipool::MultiPoolPanel, pool::pool_type::PoolTypeExt},
    wm::edit_window::EditWindowDelegate,
};

mod actions {
    use gpui::{App, KeyBinding};

    pub const CONTEXT: &str = "demex-add-pool-window";

    gpui::actions!([QuitAddPoolWindow]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new("escape", QuitAddPoolWindow, Some(CONTEXT))]);
    }
}

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

#[derive(Clone)]
struct PoolTypeSelectItem {
    title: SharedString,
    value: PoolType,
}

impl From<PoolType> for PoolTypeSelectItem {
    fn from(value: PoolType) -> Self {
        Self {
            title: value.to_string().into(),
            value,
        }
    }
}

impl SelectItem for PoolTypeSelectItem {
    type Value = PoolType;

    fn title(&self) -> gpui::SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

pub struct AddPoolWindow {
    pool: Entity<MultiPoolPanel>,

    select_state: Entity<SelectState<Vec<PoolTypeSelectItem>>>,
    input_state: Entity<InputState>,

    start_cell: (u16, u16),
    end_cell: (u16, u16),

    _subscriptions: Vec<Subscription>,
}

impl AddPoolWindow {
    pub fn new(
        pool: Entity<MultiPoolPanel>,
        start_cell: (u16, u16),
        end_cell: (u16, u16),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let select_state = cx.new(|cx| {
            SelectState::new(
                PoolType::all().into_iter().map_into().collect(),
                None,
                window,
                cx,
            )
        });

        let input_state = cx.new(|cx| InputState::new(window, cx).default_value("0"));

        let _subscriptions = vec![
            cx.subscribe(
                &select_state,
                |this, _, evt: &SelectEvent<Vec<PoolTypeSelectItem>>, cx| match evt {
                    SelectEvent::Confirm(_) => {
                        this.set_edited(true, cx);
                    }
                },
            ),
            cx.subscribe(&input_state, |this, _, evt: &InputEvent, cx| match evt {
                InputEvent::Change => {
                    this.set_edited(true, cx);
                }
                _ => {}
            }),
        ];

        Self {
            pool,
            select_state,
            input_state,
            start_cell,
            end_cell,
            _subscriptions,
        }
    }
}

impl Render for AddPoolWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .p_4()
            .gap_2()
            .justify_between()
            .child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .child(Select::new(&self.select_state))
                    .child(NumberInput::new(&self.input_state).placeholder("Start ID")),
            )
            .child(
                Button::new("save")
                    .w_full()
                    .label("Ok")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.handle_save(window, cx);
                        this.discard_and_close(cx);
                    })),
            )
    }
}

impl EditWindowDelegate for AddPoolWindow {
    fn window_title(
        &self,
        _window: &mut gpui::Window,
        _cx: &gpui::App,
    ) -> impl Into<gpui::SharedString> {
        "Add Pool"
    }

    fn handle_save(&self, _window: &mut gpui::Window, cx: &mut gpui::App) {
        if let Some((pool_type, start_id)) = self
            .select_state
            .read(cx)
            .selected_value()
            .copied()
            .zip(self.input_state.read(cx).value().parse::<u32>().ok())
        {
            self.pool.update(cx, |pool, cx| {
                pool.add_pool(pool_type, self.start_cell, self.end_cell, start_id, cx);
            });
        }
    }

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}

    fn window_bounds(cx: &mut App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(size(600.0.into(), 400.0.into()), cx))
    }
}
