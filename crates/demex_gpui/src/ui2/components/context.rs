use std::rc::Rc;

use gpui::{
    App, Context, Div, Entity, InteractiveElement, IntoElement, ParentElement, Pixels, Point,
    Render, SharedString, Styled, Window, div, px, size,
};
use gpui_component::{
    ActiveTheme,
    button::{Button, ButtonVariant, ButtonVariants},
    v_flex,
};
use itertools::Itertools;

use crate::ui2::{
    panels::pool::pool_quick_actions::{PoolQuickActions, PoolQuickActionsState},
    wm::WindowManager,
};

#[derive(Clone)]
pub struct DemexContextMenuAction {
    label: SharedString,
    action: Rc<dyn Fn(&mut Window, &mut App)>,
}

impl<L, A> From<(L, A)> for DemexContextMenuAction
where
    L: Into<SharedString>,
    A: Fn(&mut Window, &mut App) + 'static,
{
    fn from((label, action): (L, A)) -> Self {
        DemexContextMenuAction {
            label: label.into(),
            action: Rc::new(action),
        }
    }
}

#[derive(Clone)]
pub enum DemexContextLayerContentMode {
    QuickActions {
        state: Entity<PoolQuickActionsState>,
    },

    ContextMenu {
        actions: Vec<DemexContextMenuAction>,
    },

    Test,
}

#[derive(Clone)]
pub struct DemexContextLayerContent {
    pub(crate) content_mode: DemexContextLayerContentMode,
    pub(crate) pos: Point<Pixels>,
}

impl DemexContextLayerContent {
    fn positioned_div(&self, w: Pixels, h: Pixels) -> Div {
        div()
            .absolute()
            .top(self.pos.y - w / 2.0)
            .left(self.pos.x - h / 2.0)
            .w(w)
            .h(h)
    }

    pub fn render(&self, window: &mut Window, cx: &App) -> impl IntoElement {
        match &self.content_mode {
            DemexContextLayerContentMode::QuickActions { state } => (PoolQuickActions {
                state: state.clone(),
            })
            .into_any_element(),
            DemexContextLayerContentMode::ContextMenu { actions } => {
                let window_bounds = window.inner_window_bounds().get_bounds();

                let button_height = px(24.0);

                let padding = size(px(0.0), px(8.0));
                let width = px(160.0) + padding.width * 2.0;
                let height = (actions.len() as f32 * button_height) + padding.height * 2.0;

                let x = self.pos.x - ((self.pos.x + width) - window_bounds.size.width).max(px(0.0));
                let y =
                    self.pos.y - ((self.pos.y + height) - window_bounds.size.height).max(px(0.0));

                v_flex()
                    .absolute()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .left(x)
                    .top(y)
                    .w(width)
                    .h(height)
                    .px(padding.width)
                    .py(padding.height)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_md()
                    .children(actions.iter().enumerate().map(|(idx, action)| {
                        let on_click = action.action.clone();

                        Button::new(("context-menu-action", idx))
                            .w_full()
                            .h(button_height)
                            .with_variant(ButtonVariant::Secondary)
                            .child(div().size_full().text_sm().child(action.label.clone()))
                            .on_click(move |_, window, cx| {
                                let window_handle = window.window_handle();

                                cx.defer(move |cx| {
                                    let _ = WindowManager::update_dock_window_handle(
                                        window_handle,
                                        cx,
                                        |dw, _, cx| {
                                            dw.context_layer()
                                                .update(cx, |context, cx| context.clear(cx));
                                        },
                                    );
                                });

                                on_click(window, cx);
                            })
                    }))
                    .into_any_element()
            }
            DemexContextLayerContentMode::Test => (self
                .positioned_div(100.0.into(), 100.0.into())
                .bg(cx.theme().red)
                .into_element())
            .into_any_element(),
        }
    }
}

#[derive(Clone, Default)]
pub struct DemexContextLayer {
    content: Option<DemexContextLayerContent>,
}

impl DemexContextLayer {
    pub fn set_content(&mut self, content: DemexContextLayerContent, cx: &mut Context<Self>) {
        self.content = Some(content);
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.content = None;
        cx.notify();
    }

    pub fn context_menu<I, A>(&mut self, pos: Point<Pixels>, actions: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = A>,
        A: Into<DemexContextMenuAction>,
    {
        self.set_content(
            DemexContextLayerContent {
                content_mode: DemexContextLayerContentMode::ContextMenu {
                    actions: actions.into_iter().map_into().collect(),
                },
                pos,
            },
            cx,
        );
    }

    pub fn test(&mut self, window: &Window, cx: &mut Context<Self>) {
        let center = window.bounds().center();
        self.set_content(
            DemexContextLayerContent {
                content_mode: DemexContextLayerContentMode::Test,
                pos: center,
            },
            cx,
        );
    }
}

impl Render for DemexContextLayer {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let Some(content) = self.content.as_ref() else {
            return div();
        };

        div()
            .on_any_mouse_down(cx.listener(|this, _, _, cx| {
                if let Some(content) = this.content.as_ref() {
                    match &content.content_mode {
                        DemexContextLayerContentMode::ContextMenu { .. } => {
                            this.content = None;
                            cx.notify();
                        }
                        _ => {}
                    }
                }
            }))
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(content.render(window, cx))
    }
}
