use std::rc::Rc;

use gpui::{
    App, ClickEvent, Div, ElementId, Entity, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, Stateful, StatefulInteractiveElement, StyleRefinement,
    Styled, Window, canvas, div, prelude::FluentBuilder,
};
use gpui_component::{ActiveTheme, Colorize, Disableable, StyledExt, v_flex};

use crate::ui2::panels::pool::pool_quick_actions::{
    PoolQuickAction, PoolQuickActionsState, PoolQuickActionsStateEntityExtension,
};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub enum PoolItemButtonIndicatorColor {
    #[default]
    Black,
    Red,
    Green,
    Blue,
}

impl PoolItemButtonIndicatorColor {
    pub fn color(self, cx: &App) -> Hsla {
        match self {
            Self::Black => cx.theme().accent,
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
    item_id: Option<u32>,
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
            item_id: None,
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

    pub fn item_id(mut self, item_id: u32) -> Self {
        self.item_id = Some(item_id);
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

            move |evt, window, cx| {
                let quick_actions = quick_actions.clone();
                let id = id.clone();

                state.mouse_down(evt.position, quick_actions, id, window, cx);
            }
        })
        .on_mouse_up(MouseButton::Left, {
            let state = state.clone();
            let id = id.clone();

            move |evt, window, cx| {
                let id = id.clone();
                state.mouse_up(evt.position, id, window, cx);
            }
        })
        .on_mouse_move({
            let state = state.clone();
            let id = id.clone();

            move |evt, window, cx| {
                let id = id.clone();
                state.mouse_move(evt.position, id, window, cx);
            }
        })
        .on_mouse_up_out(MouseButton::Left, {
            let state = state.clone();
            let id = id.clone();

            move |evt, window, cx| {
                let id = id.clone();
                state.mouse_up(evt.position, id, window, cx);
            }
        })
    }
}

impl RenderOnce for PoolButton {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl gpui::IntoElement {
        let lighten_factor = 0.5;

        self.base
            .relative()
            .child(
                canvas(
                    {
                        let quick_actions_state = self.quick_actions_state.clone();
                        let id = self.id.clone();

                        move |bounds, _, cx| {
                            if let Some(state) = quick_actions_state {
                                state.update(cx, |state, _| {
                                    state.update_bounds(id, bounds);
                                });
                            }
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
            .when(!self.disabled, |this| {
                this.bg(cx.theme().secondary.lighten(lighten_factor))
                    .hover(|this| this.bg(cx.theme().secondary_hover.lighten(lighten_factor)))
                    .active(|this| this.bg(cx.theme().secondary_active.lighten(lighten_factor)))
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
                        .p_2()
                        .overflow_hidden()
                        .justify_center()
                        .items_center()
                        .font_semibold()
                        .text_sm()
                        .text_ellipsis()
                        .child(item_name),
                )
            })
            .child(
                div()
                    .w_full()
                    .h_2()
                    .bg(self.indicator_color.unwrap_or_default().color(cx)),
            )
            .when_some(self.item_id, |this, item_id| {
                this.child(
                    div()
                        .w_full()
                        .px_1()
                        .text_xs()
                        .font_semibold()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{}", item_id)),
                )
            })
            .when_some(
                (!self.disabled)
                    .then_some(self.quick_actions_state)
                    .flatten(),
                |div, state| {
                    Self::when_quick_actions_state(div, state, self.quick_actions, self.id)
                },
            )
    }
}
