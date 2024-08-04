use eframe::egui::{Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};

use super::DemexUiContext;

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
}

pub struct DemexTabViewer<'a> {
    context: &'a mut DemexUiContext,
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

impl DemexTabs {
    pub fn new() -> Self {
        let tabs = vec![
            DemexTab::FixtureList,
            DemexTab::PresetGrid,
            DemexTab::LayoutView,
            DemexTab::FixtureControls,
        ];

        let dock_state = DockState::new(tabs);
        Self { dock_state }
    }

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
