use eframe::egui::{Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::ui::tabs::timecode_clock_tab::ClockComponent;

use super::DemexUiContext;

pub mod color_picker_tab;
pub mod empty_tab;
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
pub mod timecode_clock_tab;
pub mod timing_tab;

pub struct DemexAddedTab {
    tab: DemexTab,
    surface: egui_dock::SurfaceIndex,
    node: egui_dock::NodeIndex,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize, EnumIter)]
pub enum DemexTab {
    LayoutView,
    FixtureList,
    PresetGrid,
    Faders,
    SequencesList,
    SequenceEditor,
    Encoders,
    TimecodeClock,
    Patch,
    Timing,
    FixtureSelection,
    ColorPicker,
    Logs,
    Performance,
    Empty,
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
            DemexTab::TimecodeClock => write!(f, "Timecode Clock"),
            DemexTab::Patch => write!(f, "Patch"),
            DemexTab::Timing => write!(f, "Timing"),
            DemexTab::FixtureSelection => write!(f, "Fixture Selection"),
            DemexTab::ColorPicker => write!(f, "Color Picker"),
            DemexTab::Logs => write!(f, "Logs"),
            DemexTab::Performance => write!(f, "Performance"),
            DemexTab::Empty => write!(f, "demex"),
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
                sequence_editor_tab::SequenceEditorTab::new(context).show(ui)
            }
            DemexTab::Encoders => encoders_tab::ui(ui, context),
            DemexTab::TimecodeClock => ClockComponent::new("DemexClockComponent", context).show(ui),
            DemexTab::Patch => {
                let mut patch_view_component = patch_tab::PatchViewComponent::new(context);
                patch_view_component.show(ui);
            }
            DemexTab::Timing => timing_tab::ui(ui, context),
            DemexTab::FixtureSelection => fixture_selection_tab::ui(ui, context),
            DemexTab::ColorPicker => color_picker_tab::ColorPickerComponent::new(context).show(ui),
            DemexTab::Logs => logs_tab::ui(ui, context),
            DemexTab::Performance => performance_tab::ui(ui, context),
            DemexTab::Empty => empty_tab::ui(ui, context),
        }
    }
}

pub struct DemexTabViewer<'a> {
    context: &'a mut DemexUiContext,
    allow_windows: bool,
    added_nodes: &'a mut Vec<DemexAddedTab>,
}

impl TabViewer for DemexTabViewer<'_> {
    type Tab = DemexTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        tab.ui(ui, self.context)
    }

    fn add_popup(
        &mut self,
        ui: &mut Ui,
        surface: egui_dock::SurfaceIndex,
        node: egui_dock::NodeIndex,
    ) {
        ui.set_min_width(120.0);
        ui.style_mut().visuals.button_frame = false;

        for tab in DemexTab::iter() {
            if ui.button(tab.to_string()).clicked() {
                self.added_nodes.push(DemexAddedTab { tab, surface, node });
            }
        }
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        self.allow_windows
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemexTabs {
    dock_state: DockState<DemexTab>,
}

impl DemexTabs {
    pub fn main() -> Self {
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

        let [old_node, layout_view_node] = surface.split_left(
            old_node,
            0.65,
            vec![
                DemexTab::LayoutView,
                DemexTab::FixtureSelection,
                DemexTab::SequenceEditor,
                DemexTab::Patch,
            ],
        );

        {
            let [_, new_node] =
                surface.split_left(layout_view_node, 0.3, vec![DemexTab::ColorPicker]);
            surface.split_below(new_node, 0.65, vec![DemexTab::Timing]);
        }

        surface.split_above(
            old_node,
            0.5,
            vec![
                DemexTab::PresetGrid,
                DemexTab::Faders,
                DemexTab::TimecodeClock,
            ],
        );

        Self { dock_state }
    }

    pub fn is_empty(&self) -> bool {
        self.dock_state.surfaces_count() == 0
    }

    pub fn focus(&mut self, tab: DemexTab) -> bool {
        if let Some(tab) = self.dock_state.find_tab(&tab) {
            self.dock_state.set_active_tab(tab);
            true
        } else {
            false
        }
    }
}

impl Default for DemexTabs {
    fn default() -> Self {
        Self {
            dock_state: DockState::new(vec![DemexTab::Empty]),
        }
    }
}

impl DemexTabs {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        context: &mut DemexUiContext,
        id: egui::Id,
        allow_windows: bool,
    ) {
        let mut added_nodes = Vec::new();

        DockArea::new(&mut self.dock_state)
            .id(id)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show_close_buttons(true)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(
                ui,
                &mut DemexTabViewer {
                    context,
                    allow_windows,
                    added_nodes: &mut added_nodes,
                },
            );

        added_nodes.drain(..).for_each(|tab| {
            self.dock_state
                .set_focused_node_and_surface((tab.surface, tab.node));
            self.dock_state.push_to_focused_leaf(tab.tab);
        });
    }
}
