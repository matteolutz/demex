use std::rc::Rc;

use gpui::{
    App, ClickEvent, Div, ElementId, Entity, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, Stateful, StatefulInteractiveElement, StyleRefinement,
    Styled, Window, div, prelude::FluentBuilder,
};
use gpui_component::{ActiveTheme, Disableable, v_flex};

use crate::ui2::panels::pool::pool_quick_actions::{PoolQuickAction, PoolQuickActionsState};

pub enum PoolItemButtonIndicatorColor {
    Red,
    Green,
    Blue,
}

impl PoolItemButtonIndicatorColor {
    pub fn color(self, cx: &App) -> Hsla {
        match self {
            Self::Red => cx.theme().red,
            Self::Green => cx.theme().green,
            Self::Blue => cx.theme().blue,
        }
    }
}

#[derive(IntoElement)]
pub struct PoolButton {
    id: ElementId,
    base: Stateful<Div>,

    item_name: Option<SharedString>,
    indicator_color: Option<PoolItemButtonIndicatorColor>,

    quick_actions_state: Option<Entity<PoolQuickActionsState>>,
    quick_actions: Vec<PoolQuickAction>,

    disabled: bool,

    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
}

impl PoolButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();

        Self {
            id: id.clone(),
            base: div().id(id),
            item_name: None,
            indicator_color: None,
            quick_actions_state: None,
            quick_actions: Vec::new(),
            disabled: false,
            on_click: None,
        }
    }

    pub fn item_name(mut self, name: impl Into<SharedString>) -> Self {
        self.item_name = Some(name.into());
        self
    }

    pub fn indicator_color(mut self, color: PoolItemButtonIndicatorColor) -> Self {
        self.indicator_color = Some(color);
        self
    }

    pub fn quick_actions_state(mut self, state: &Entity<PoolQuickActionsState>) -> Self {
        self.quick_actions_state = Some(state.clone());
        self
    }

    pub fn action(
        mut self,
        name: impl Into<SharedString>,
        action: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.quick_actions.push(PoolQuickAction {
            name: name.into(),
            action: Rc::new(action),
        });
        self
    }

    pub fn on_click(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }
}

impl Disableable for PoolButton {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Styled for PoolButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl PoolButton {
    fn when_quick_actions_state(
        div: Stateful<Div>,
        state: Entity<PoolQuickActionsState>,
        quick_actions: Vec<PoolQuickAction>,
        id: ElementId,
    ) -> Stateful<Div> {
        div.on_mouse_down(MouseButton::Left, {
            let state = state.clone();
            let id = id.clone();

            move |evt, _, cx| {
                let quick_actions = quick_actions.clone();
                let id = id.clone();

                state.update(cx, |state, cx| {
                    state.mouse_down(evt.position, quick_actions, id, cx);
                })
            }
        })
        .on_mouse_up(MouseButton::Left, {
            let state = state.clone();
            let id = id.clone();

            move |evt, _, cx| {
                let id = id.clone();
                state.update(cx, |state, cx| {
                    state.mouse_up(evt.position, id);
                    cx.notify();
                })
            }
        })
        .on_mouse_move({
            let state = state.clone();
            let id = id.clone();

            move |_, _, cx| {
                let id = id.clone();
                state.update(cx, |state, cx| {
                    state.mouse_move(id);
                    cx.notify();
                })
            }
        })
        .on_mouse_up_out(MouseButton::Left, {
            let state = state.clone();
            let id = id.clone();

            move |evt, _, cx| {
                let id = id.clone();
                state.update(cx, |state, cx| {
                    state.mouse_up(evt.position, id);
                    cx.notify();
                })
            }
        })
    }
}

impl RenderOnce for PoolButton {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl gpui::IntoElement {
        self.base
            .relative()
            .when(!self.disabled, |this| {
                this.bg(cx.theme().secondary)
                    .hover(|this| this.bg(cx.theme().secondary_hover))
                    .active(|this| this.bg(cx.theme().secondary_active))
            })
            .when_some(self.on_click, |this, on_click| {
                this.on_click(move |event, window, cx| {
                    if self.disabled {
                        cx.stop_propagation();
                        return;
                    }

                    (on_click)(event, window, cx)
                })
            })
            .when_some(self.item_name, |this, item_name| {
                this.child(
                    v_flex()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .justify_center()
                        .items_center()
                        .child(item_name),
                )
            })
            .when_some(self.indicator_color, |this, color| {
                this.child(div().w_full().h_2().bg(color.color(cx)))
            })
            .when_some(self.quick_actions_state, |div, state| {
                Self::when_quick_actions_state(div, state, self.quick_actions, self.id)
            })
    }
}
