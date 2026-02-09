use std::rc::Rc;

use gpui::{
    App, Bounds, ClickEvent, Div, ElementId, Entity, Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, PathBuilder, RenderOnce, Rgba, SharedString, Stateful,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, canvas, div, fill, point,
    prelude::FluentBuilder, px, size,
};
use gpui_component::{ActiveTheme, Colorize, Disableable, StyledExt, v_flex};
use itertools::Itertools;

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
    quick_actions: Vec<Option<PoolQuickAction>>,

    colors: Option<Vec<Hsla>>,

    top_right: Option<SharedString>,

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
            colors: None,
            top_right: None,
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
        self.quick_actions.push(Some(PoolQuickAction {
            name: name.into(),
            action: Rc::new(action),
        }));
        self
    }

    pub fn action_at(
        mut self,
        idx: usize,
        name: impl Into<SharedString>,
        action: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        if self.quick_actions.len() <= idx {
            self.quick_actions.resize(idx + 1, None);
        }

        self.quick_actions[idx] = Some(PoolQuickAction {
            name: name.into(),
            action: Rc::new(action),
        });

        self
    }

    pub fn top_right(mut self, annotation: impl Into<SharedString>) -> Self {
        self.top_right = Some(annotation.into());
        self
    }

    pub fn on_click(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }

    pub fn colors<I, C>(mut self, colors: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Hsla>,
    {
        self.colors = Some(colors.into_iter().map_into().collect());
        self
    }

    pub fn colors_rgb(self, colors: impl IntoIterator<Item = [f32; 3]>) -> Self {
        self.colors(colors.into_iter().map(|[r, g, b]| Rgba { r, g, b, a: 1.0 }))
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
        quick_actions: Vec<Option<PoolQuickAction>>,
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

        // let colors = self.colors;

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
                    |bounds, _, window, _| {
                        if let Some(colors) = self.colors {
                            let center = bounds.center();
                            let radius = bounds.size.width / 4.0;

                            if colors.len() > 1 {
                                let deg_per_color = 360.0 / colors.len() as f32;

                                for (idx, color) in colors.into_iter().enumerate() {
                                    let mut path_builder = PathBuilder::fill();

                                    let from = center
                                        + point(
                                            radius
                                                * (idx as f32 * deg_per_color).to_radians().cos(),
                                            radius
                                                * (idx as f32 * deg_per_color).to_radians().sin(),
                                        );

                                    let to = center
                                        + point(
                                            radius
                                                * ((idx + 1) as f32 * deg_per_color)
                                                    .to_radians()
                                                    .cos(),
                                            radius
                                                * ((idx + 1) as f32 * deg_per_color)
                                                    .to_radians()
                                                    .sin(),
                                        );

                                    path_builder.move_to(center);
                                    path_builder.line_to(from);
                                    path_builder.arc_to(
                                        point(radius, radius),
                                        px(deg_per_color),
                                        false,
                                        true,
                                        to,
                                    );

                                    window.paint_path(path_builder.build().unwrap(), color);
                                }
                            } else {
                                let circle_bounds =
                                    Bounds::centered_at(center, size(radius * 2, radius * 2));
                                window.paint_quad(
                                    fill(circle_bounds, colors[0]).corner_radii(radius),
                                );
                            }
                        }
                    },
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
                        // cx.stop_propagation();
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
                        .text_xs()
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
            .when_some(self.top_right, |this, top_right| {
                this.child(
                    div()
                        .absolute()
                        .top_2()
                        .right_1()
                        .h_4()
                        .gap_1()
                        .text_color(cx.theme().muted_foreground)
                        .text_xs()
                        .child(top_right),
                )
            })
            .when_some(self.item_id, |this, item_id| {
                this.child(
                    div()
                        .absolute()
                        .left_1()
                        .px_1()
                        .text_xs()
                        .font_semibold()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{}", item_id)),
                )
            })
            .when_some(self.quick_actions_state, |div, state| {
                Self::when_quick_actions_state(div, state, self.quick_actions, self.id)
            })
    }
}
