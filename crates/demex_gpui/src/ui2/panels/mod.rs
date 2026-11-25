use std::sync::Arc;

use gpui::{
    App, AppContext, Context, Entity, EventEmitter, Focusable, IntoElement, Render, WeakEntity,
    Window,
};
use gpui_component::{
    IconName,
    button::Button,
    dock::{
        DockArea, Panel, PanelEvent, PanelInfo, PanelState, PanelView, TabPanel, register_panel,
    },
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::ui2::{
    components::context::DemexContextMenuAction,
    panels::{
        attribute_editor::AttributeEditorPanel,
        clock::ClockPanel,
        color_picker::ColorPickerPanel,
        command::CommandPanel,
        fixture_list::FixtureListPanel,
        fixture_selection::FixtureSelectionPanel,
        layout_view::LayoutViewPanel,
        multipool::{MultiPoolPanel, config::MultiPoolConfig},
        performance::PerformancePanel,
        sequence_editor::SequenceEditorPanel,
        speedmaster::SpeedmasterPanel,
    },
    utils::profiler::DemexUiProfilerGlobalBorrowAppExt,
    wm::WindowManager,
};

pub mod attribute_editor;
pub mod clock;
pub mod color_picker;
pub mod command;
pub mod fixture_list;
pub mod fixture_selection;
pub mod layout_view;
pub mod multipool;
pub mod performance;
pub mod pool;
pub mod sequence_editor;
pub mod speedmaster;

mod actions {
    gpui::actions!(panels, [AddFixtureSelection, AddFixtureList, AddLayoutView]);
}

#[derive(
    Debug, Copy, Clone, strum::EnumIter, strum::Display, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum DockWindowPanelType {
    AttributeEditor,
    FixtureList,
    FixtureSelection,
    LayoutView,
    Multipool,
    SequenceEditor,
    ColorPicker,
    Clock,
    Speedmaster,

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
            DockWindowPanelType::ColorPicker => "demex-color-picker",
            DockWindowPanelType::Clock => "demex-clock",
            DockWindowPanelType::Speedmaster => "demex-speedmaster",

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
            | Self::SequenceEditor
            | Self::ColorPicker
            | Self::Clock
            | Self::Speedmaster => true,

            Self::PerformancePanel => true,

            Self::CommandPanel => false,
        }
    }

    pub fn iter_creatable() -> impl Iterator<Item = Self> {
        Self::iter().filter(|panel_type| panel_type.allow_creating())
    }

    pub fn build_panel_view(self, window: &mut Window, cx: &mut App) -> Arc<dyn PanelView> {
        match self {
            DockWindowPanelType::AttributeEditor => {
                Arc::new(cx.new_panel(|cx| AttributeEditorPanel::new(window, cx)))
            }
            DockWindowPanelType::FixtureList => {
                Arc::new(cx.new_panel(|cx| FixtureListPanel::new(window, cx)))
            }
            DockWindowPanelType::FixtureSelection => {
                Arc::new(cx.new_panel(|cx| FixtureSelectionPanel::new(cx)))
            }
            DockWindowPanelType::Multipool => {
                Arc::new(cx.new_panel(|cx| MultiPoolPanel::new(MultiPoolConfig::default(), cx)))
            }
            DockWindowPanelType::LayoutView => {
                Arc::new(cx.new_panel(|cx| LayoutViewPanel::new(window, cx)))
            }
            DockWindowPanelType::SequenceEditor => {
                Arc::new(cx.new_panel(|cx| SequenceEditorPanel::new(window, cx)))
            }
            DockWindowPanelType::ColorPicker => {
                Arc::new(cx.new_panel(|cx| ColorPickerPanel::new(window, cx)))
            }
            DockWindowPanelType::Clock => Arc::new(cx.new_panel(|cx| ClockPanel::new(window, cx))),
            DockWindowPanelType::Speedmaster => {
                Arc::new(cx.new_panel(|cx| SpeedmasterPanel::new(window, cx)))
            }
            DockWindowPanelType::CommandPanel => {
                Arc::new(cx.new_panel(|cx| CommandPanel::new(window, cx)))
            }
            DockWindowPanelType::PerformancePanel => {
                Arc::new(cx.new_panel(|cx| PerformancePanel::new(window, cx)))
            }
        }
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        cx.with_profiler(T::panel_type().into(), |_| self.view.clone())
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
        let current_parent = self.parent.clone();

        Some(vec![
            Button::new("add-panel")
                .icon(IconName::Plus)
                .on_click(move |evt, window, cx| {
                    let current_parent = current_parent.clone();
                    let window_handle = window.window_handle();
                    let pos = evt.position();

                    let context_menu_actions: Vec<DemexContextMenuAction> =
                        DockWindowPanelType::iter_creatable()
                            .map(|panel_type| {
                                let current_parent = current_parent.clone();

                                (
                                    panel_type.to_string(),
                                    move |window: &mut Window, cx: &mut App| {
                                        let current_parent = current_parent.clone();

                                        let window_handle = window.window_handle();
                                        window.defer(cx, move |window, cx| {
                                            if let Some(current_parent) = current_parent {
                                                let _ = current_parent.update(cx, |parent, cx| {
                                                    parent.add_panel(
                                                        panel_type.build_panel_view(window, cx),
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            } else {
                                                let _ = WindowManager::update_dock_window_handle(
                                                    window_handle,
                                                    cx,
                                                    |dw, window, cx| {
                                                        dw.add_panel(panel_type, window, cx)
                                                    },
                                                );
                                            }
                                        });
                                    },
                                )
                            })
                            .map_into()
                            .collect::<Vec<_>>();

                    cx.defer(move |cx| {
                        let current_parent = current_parent.clone();

                        let _ = WindowManager::update_dock_window_handle(
                            window_handle,
                            cx,
                            move |dw, _, cx| {
                                let current_parent = current_parent.clone();

                                dw.context_layer().update(cx, move |context, cx| {
                                    let current_parent = current_parent.clone();
                                    context.context_menu(pos, context_menu_actions, cx);
                                });
                            },
                        );
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
    ColorPickerPanel::register(cx);
    ClockPanel::register(cx);
    SpeedmasterPanel::register(cx);
}
