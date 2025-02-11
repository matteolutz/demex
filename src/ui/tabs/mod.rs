use std::collections::HashSet;

use eframe::egui::{Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};

use super::DemexUiContext;

pub mod faders_tab;
pub mod fixture_controls_tab;
pub mod fixture_list_tab;
pub mod layout_view_tab;
pub mod logs_tab;
pub mod performance_tab;
pub mod preset_grid_tab;
pub mod sequences_list_tab;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum DemexTab {
    LayoutView,
    FixtureList,
    PresetGrid,
    FixtureControls,
    Faders,
    SequencesList,
    Logs,
    Performance,
}

impl std::fmt::Display for DemexTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DemexTab::LayoutView => write!(f, "Layout View"),
            DemexTab::FixtureList => write!(f, "Fixture List"),
            DemexTab::PresetGrid => write!(f, "Preset Grid"),
            DemexTab::FixtureControls => write!(f, "Fixture Controls"),
            DemexTab::Faders => write!(f, "Faders"),
            DemexTab::SequencesList => write!(f, "Sequences List"),
            DemexTab::Logs => write!(f, "Logs"),
            DemexTab::Performance => write!(f, "Performance"),
        }
    }
}

impl DemexTab {
    pub fn ui(&self, ui: &mut Ui, context: &mut DemexUiContext) {
        match self {
            DemexTab::LayoutView => layout_view_tab::ui(ui, context),
            DemexTab::FixtureList => fixture_list_tab::ui(ui, context),
            DemexTab::PresetGrid => preset_grid_tab::ui(ui, context),
            DemexTab::FixtureControls => fixture_controls_tab::ui(ui, context),
            DemexTab::Faders => faders_tab::ui(ui, context),
            DemexTab::SequencesList => sequences_list_tab::ui(ui, context),
            DemexTab::Logs => logs_tab::ui(ui, context),
            DemexTab::Performance => performance_tab::ui(ui, context),
        }
    }
}

pub struct DemexTabViewer<'a> {
    context: &'a mut DemexUiContext,
    #[allow(dead_code)]
    egui_ctx: &'a eframe::egui::Context,
    detached_tabs: &'a mut HashSet<DemexTab>,
}

impl<'a> TabViewer for DemexTabViewer<'a> {
    type Tab = DemexTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        format!("{:?}", tab).into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        tab.ui(ui, self.context)
    }

    fn context_menu(
        &mut self,
        ui: &mut Ui,
        tab: &mut Self::Tab,
        _surface: egui_dock::SurfaceIndex,
        _node: egui_dock::NodeIndex,
    ) {
        if ui.button("Window").clicked() {
            self.detached_tabs.insert(*tab);
        }
    }

    fn closeable(&mut self, _: &mut Self::Tab) -> bool {
        false
    }
}

pub struct DemexTabs {
    dock_state: DockState<DemexTab>,
}

impl Default for DemexTabs {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![
            DemexTab::FixtureList,
            DemexTab::SequencesList,
            DemexTab::Logs,
            DemexTab::Performance,
        ]);

        let surface = dock_state.main_surface_mut();
        let [old_node, new_node] = surface.split_left(
            egui_dock::NodeIndex::root(),
            0.65,
            vec![DemexTab::FixtureControls],
        );

        surface.split_below(new_node, 0.5, vec![DemexTab::LayoutView]);
        surface.split_above(old_node, 0.5, vec![DemexTab::PresetGrid, DemexTab::Faders]);

        Self { dock_state }
    }
}

impl DemexTabs {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        context: &mut DemexUiContext,
        egui_ctx: &eframe::egui::Context,
        detached_tabs: &mut HashSet<DemexTab>,
    ) {
        self.dock_state = self.dock_state.filter_tabs(|f| !detached_tabs.contains(f));

        DockArea::new(&mut self.dock_state)
            .show_add_buttons(true)
            .show_close_buttons(false)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(
                ui,
                &mut DemexTabViewer {
                    context,
                    egui_ctx,
                    detached_tabs,
                },
            );
    }

    pub fn re_attach(&mut self, tab: DemexTab) {
        self.dock_state.add_window(vec![tab]);
    }
}
