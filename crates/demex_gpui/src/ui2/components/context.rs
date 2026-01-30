use gpui::{
    App, Context, Div, Entity, IntoElement, ParentElement, Pixels, Point, Render, Styled, Window,
    div,
};
use gpui_component::{ActiveTheme, StyledExt};

use crate::ui2::panels::pool::pool_quick_actions::{PoolQuickActions, PoolQuickActionsState};

#[derive(Clone)]
pub enum DemexContextLayerContentMode {
    QuickActions {
        state: Entity<PoolQuickActionsState>,
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

    pub fn render(&self, cx: &App) -> impl IntoElement {
        log::debug!("rendering context layer content");

        match &self.content_mode {
            DemexContextLayerContentMode::QuickActions { state } => (PoolQuickActions {
                state: state.clone(),
            })
            .into_any_element(),
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

    #[cfg(debug_assertions)]
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
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let Some(content) = self.content.as_ref() else {
            return div();
        };

        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(content.render(cx))
    }
}
