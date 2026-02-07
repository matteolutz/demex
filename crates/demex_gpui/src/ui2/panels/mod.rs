use gpui::{
    App, AppContext, Context, Entity, EventEmitter, Focusable, IntoElement, Render, WeakEntity,
    Window,
};
use gpui_component::{
    IconName,
    button::Button,
    dock::{DockArea, Panel, PanelEvent, PanelInfo, PanelState, TabPanel, register_panel},
};
use strum::IntoEnumIterator;

use crate::ui2::{
    panels::{
        attribute_editor::AttributeEditorPanel, command::CommandPanel,
        fixture_list::FixtureListPanel, fixture_selection::FixtureSelectionPanel,
        layout_view::LayoutViewPanel, multipool::MultiPoolPanel, performance::PerformancePanel,
        sequence_editor::SequenceEditorPanel,
    },
    wm::WindowManager,
};

pub mod attribute_editor;
pub mod command;
pub mod fixture_list;
pub mod fixture_selection;
pub mod layout_view;
pub mod multipool;
pub mod performance;
pub mod pool;
pub mod sequence_editor;

mod actions {
    gpui::actions!(panels, [AddFixtureSelection, AddFixtureList, AddLayoutView]);
}

#[derive(Debug, Copy, Clone, strum::EnumIter, strum::Display)]
pub enum DockWindowPanelType {
    AttributeEditor,
    FixtureList,
    FixtureSelection,
    LayoutView,
    Multipool,
    SequenceEditor,

    CommandPanel,
    PerformancePanel,
}

impl DockWindowPanelType {
    pub fn panel_name(&self) -> &'static str {
        match self {
            DockWindowPanelType::AttributeEditor => "demex-attribute-editor",
            DockWindowPanelType::FixtureList => "demex-fixture-list",
            DockWindowPanelType::FixtureSelection => "demex-fixture-selection",
            DockWindowPanelType::LayoutView => "layout-view",
            DockWindowPanelType::Multipool => "demex-multipool",
            DockWindowPanelType::SequenceEditor => "demex-sequence-editor",

            DockWindowPanelType::CommandPanel => "demex-command",
            DockWindowPanelType::PerformancePanel => "demex-performance",
        }
    }

    pub fn allow_creating(&self) -> bool {
        match self {
            Self::AttributeEditor
            | Self::FixtureList
            | Self::FixtureSelection
            | Self::LayoutView
            | Self::Multipool
            | Self::SequenceEditor => true,

            Self::CommandPanel | Self::PerformancePanel => false,
        }
    }

    pub fn iter_creatable() -> impl Iterator<Item = Self> {
        Self::iter().filter(|panel_type| panel_type.allow_creating())
    }
}

pub struct DemexPanelView<T: DemexPanel> {
    view: Entity<T>,
    parent: Option<WeakEntity<TabPanel>>,
}

impl<T: DemexPanel> DemexPanelView<T> {
    pub fn new(build_view: impl FnOnce(&mut Context<T>) -> T, cx: &mut Context<Self>) -> Self {
        Self {
            view: cx.new(|cx| build_view(cx)),
            parent: None,
        }
    }
}

#[allow(unused_variables)]
pub trait DemexPanel: EventEmitter<PanelEvent> + Render + Focusable {
    fn panel_type() -> DockWindowPanelType;

    fn deserialize(
        dock_area: WeakEntity<DockArea>,
        panel_state: &PanelState,
        panel_info: &PanelInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self;

    fn register(cx: &mut App) {
        register_panel(
            cx,
            Self::panel_type().panel_name(),
            |dw, ps, pi, window, cx| {
                Box::new(cx.new_panel(|cx| Self::deserialize(dw, ps, pi, window, cx)))
            },
        );
    }

    fn title_suffix(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        None::<gpui::Div>
    }

    fn dump(&self, state: &mut PanelState, cx: &App) {}
}

impl<T: DemexPanel> EventEmitter<PanelEvent> for DemexPanelView<T> {}
impl<T: DemexPanel> Focusable for DemexPanelView<T> {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.view.read(cx).focus_handle(cx)
    }
}
impl<T: DemexPanel> Render for DemexPanelView<T> {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        self.view.clone()
    }
}

#[allow(unused_variables)]
impl<T: DemexPanel> Panel for DemexPanelView<T> {
    fn panel_name(&self) -> &'static str {
        T::panel_type().panel_name()
    }

    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        T::panel_type().to_string()
    }

    fn title_suffix(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        self.view.update(cx, |view, cx| {
            view.title_suffix(window, cx)
                .map(|val| val.into_any_element())
        })
    }

    fn inner_padding(&self, cx: &App) -> bool {
        false
    }

    fn dump(&self, cx: &App) -> PanelState {
        let mut state = PanelState::new(self);
        self.view.read(cx).dump(&mut state, cx);
        state
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        Some(    vec![
            Button::new("add-panel")
                .icon(IconName::Plus)
                .on_click(|evt, window, cx| {
                    let window_handle = window.window_handle();
                    let pos = evt.position();

                    cx.defer(move |cx| {
                        let _ =
                            WindowManager::update_dock_window_handle(window_handle, cx, |dw, _, cx| {
                                dw.context_layer().update(cx, |context, cx| {
                                    context.context_menu(
                                        pos,
                                        DockWindowPanelType::iter_creatable().map(|panel_type| {
                                            (
                                                panel_type.to_string(),
                                                move |window: &mut Window, cx: &mut App| {
                                                    let window_handle = window.window_handle();
                                                    cx.defer(move |cx| {
                                                        let _ =
                                                            WindowManager::update_dock_window_handle(
                                                                window_handle,
                                                                cx,
                                                                |dw, window, cx| {
                                                                    dw.add_panel(panel_type, window, cx)
                                                                },
                                                            );
                                                    });
                                                },
                                            )
                                        }),
                                        cx,
                                    );
                                });
                            });
                    });
                }),
        ])
    }

    fn on_added_to(
        &mut self,
        tab_panel: WeakEntity<TabPanel>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.parent = Some(tab_panel);
    }

    fn on_removed(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.parent = None;
    }
}

pub trait DemexPanelContextExt {
    fn new_panel<E: DemexPanel>(
        &mut self,
        build_panel: impl FnOnce(&mut Context<E>) -> E,
    ) -> Entity<DemexPanelView<E>>;
}
impl DemexPanelContextExt for App {
    fn new_panel<E: DemexPanel>(
        &mut self,
        build_panel: impl FnOnce(&mut Context<E>) -> E,
    ) -> Entity<DemexPanelView<E>> {
        self.new(|cx| DemexPanelView::new(build_panel, cx))
    }
}

pub fn init(cx: &mut App) {
    register_panels(cx);

    command::init(cx);
    multipool::init(cx);
}

fn register_panels(cx: &mut App) {
    CommandPanel::register(cx);
    FixtureSelectionPanel::register(cx);
    PerformancePanel::register(cx);
    FixtureListPanel::register(cx);
    // pool::register(cx);
    LayoutViewPanel::register(cx);
    SequenceEditorPanel::register(cx);
    AttributeEditorPanel::register(cx);
    MultiPoolPanel::register(cx);
}
