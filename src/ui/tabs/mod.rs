use eframe::egui::{Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};
use serde::{Deserialize, Serialize};

use super::DemexUiContext;

pub mod encoders_tab;
pub mod faders_tab;
pub mod fixture_list_tab;
pub mod fixture_selection_tab;
pub mod layout_view_tab;
pub mod logs_tab;
pub mod patch_tab;
pub mod performance_tab;
pub mod preset_grid_tab;
pub mod sequence_editor_tab;
pub mod sequences_list_tab;
pub mod timing_tab;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum DemexTab {
    LayoutView,
    FixtureList,
    PresetGrid,
    Faders,
    SequencesList,
    SequenceEditor,
    Encoders,
    Patch,
    Timing,
    FixtureSelection,
    Logs,
    Performance,
}

impl std::fmt::Display for DemexTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DemexTab::LayoutView => write!(f, "Layout View"),
            DemexTab::FixtureList => write!(f, "Fixture List"),
            DemexTab::PresetGrid => write!(f, "Preset Grid"),
            DemexTab::Faders => write!(f, "Faders"),
            DemexTab::SequencesList => write!(f, "Sequences List"),
            DemexTab::SequenceEditor => write!(f, "Sequence Editor"),
            DemexTab::Encoders => write!(f, "Encoders"),
            DemexTab::Patch => write!(f, "Patch"),
            DemexTab::Timing => write!(f, "Timing"),
            DemexTab::FixtureSelection => write!(f, "Fixture Selection"),
            DemexTab::Logs => write!(f, "Logs"),
            DemexTab::Performance => write!(f, "Performance"),
        }
    }
}

impl DemexTab {
    pub fn ui(&self, ui: &mut Ui, context: &mut DemexUiContext) {
        match self {
            DemexTab::LayoutView => layout_view_tab::LayoutViewComponent::new(context).show(ui),
            DemexTab::FixtureList => fixture_list_tab::ui(ui, context),
            DemexTab::PresetGrid => preset_grid_tab::ui(ui, context),
            DemexTab::Faders => faders_tab::ui(ui, context),
            DemexTab::SequencesList => sequences_list_tab::ui(ui, context),
            DemexTab::SequenceEditor => {
                sequence_editor_tab::SequenceEditorTab::new(context, "MainSequenceEditor").show(ui)
            }
            DemexTab::Encoders => encoders_tab::ui(ui, context),
            DemexTab::Patch => {
                let mut patch_view_component = patch_tab::PatchViewComponent::new(context);
                patch_view_component.show(ui);
            }
            DemexTab::Timing => timing_tab::ui(ui, context),
            DemexTab::FixtureSelection => fixture_selection_tab::ui(ui, context),
            DemexTab::Logs => logs_tab::ui(ui, context),
            DemexTab::Performance => performance_tab::ui(ui, context),
        }
    }
}

pub struct DemexTabViewer<'a> {
    context: &'a mut DemexUiContext,
    #[allow(dead_code)]
    egui_ctx: &'a eframe::egui::Context,
}

impl TabViewer for DemexTabViewer<'_> {
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
            self.context.ui_config.detached_tabs.insert(*tab);
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
            DemexTab::Performance,
        ]);

        let surface = dock_state.main_surface_mut();

        let [old_node, _] = surface.split_below(
            egui_dock::NodeIndex::root(),
            0.8,
            vec![DemexTab::Encoders, DemexTab::Logs],
        );

        let [old_node, _] = surface.split_left(
            old_node,
            0.65,
            vec![
                DemexTab::LayoutView,
                DemexTab::FixtureSelection,
                DemexTab::SequenceEditor,
                DemexTab::Patch,
            ],
        );

        surface.split_above(
            old_node,
            0.5,
            vec![DemexTab::PresetGrid, DemexTab::Faders, DemexTab::Timing],
        );

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
        self.dock_state = self
            .dock_state
            .filter_tabs(|f| !context.ui_config.detached_tabs.contains(f));

        DockArea::new(&mut self.dock_state)
            .show_add_buttons(true)
            .show_close_buttons(false)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut DemexTabViewer { context, egui_ctx });
    }

    pub fn re_attach(&mut self, tab: DemexTab) {
        self.dock_state.add_window(vec![tab]);
    }
}
