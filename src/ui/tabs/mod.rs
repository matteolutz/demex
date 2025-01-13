use eframe::egui::{Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};

use super::DemexUiContext;

pub mod debug_tab;
pub mod fixture_controls_tab;
pub mod fixture_list_tab;
pub mod layout_view_tab;
pub mod preset_grid_tab;

#[derive(Debug, PartialEq)]
pub enum DemexTab {
    LayoutView,
    FixtureList,
    PresetGrid,
    FixtureControls,
    Debug,
}

pub struct DemexTabViewer<'a> {
    context: &'a mut DemexUiContext,
    #[allow(dead_code)]
    egui_ctx: &'a eframe::egui::Context,
}

impl<'a> TabViewer for DemexTabViewer<'a> {
    type Tab = DemexTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        format!("{:?}", tab).into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            DemexTab::LayoutView => layout_view_tab::ui(ui, self.context),
            DemexTab::FixtureList => fixture_list_tab::ui(ui, self.context),
            DemexTab::PresetGrid => preset_grid_tab::ui(ui, self.context),
            DemexTab::FixtureControls => fixture_controls_tab::ui(ui, self.context),
            DemexTab::Debug => debug_tab::ui(ui, self.context),
        }
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        // prevent closing tabs
        false
    }

    fn on_add(&mut self, _surface: egui_dock::SurfaceIndex, _node: egui_dock::NodeIndex) {
        println!("on add!!!");
    }
}

pub struct DemexTabs {
    dock_state: DockState<DemexTab>,
}

impl Default for DemexTabs {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![DemexTab::FixtureList, DemexTab::Debug]);

        let surface = dock_state.main_surface_mut();
        let [old_node, new_node] = surface.split_left(
            egui_dock::NodeIndex::root(),
            0.65,
            vec![DemexTab::FixtureControls],
        );

        surface.split_below(new_node, 0.5, vec![DemexTab::LayoutView]);
        surface.split_above(old_node, 0.5, vec![DemexTab::PresetGrid]);

        Self { dock_state }
    }
}

impl DemexTabs {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        context: &mut DemexUiContext,
        egui_ctx: &eframe::egui::Context,
    ) {
        DockArea::new(&mut self.dock_state)
            .show_add_buttons(true)
            .show_close_buttons(false)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut DemexTabViewer { context, egui_ctx });
    }
}
