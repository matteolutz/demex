use strum::IntoEnumIterator;

use crate::ui::{
    components::tab_viewer::TabViewer,
    context::DemexUiContext,
    tabs::timecode_clock_tab::{mode::ClockMode, value::ClockValue},
};

pub mod mode;
pub mod value;

pub struct ClockComponent<'a> {
    id: egui::Id,
    context: &'a DemexUiContext,
}

impl<'a> ClockComponent<'a> {
    pub fn new(id: impl Into<egui::Id>, context: &'a DemexUiContext) -> Self {
        Self {
            id: id.into(),
            context,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let timing_handler = self.context.timing_handler.read();

        ui.vertical(|ui| {
            let clock_mode = TabViewer::new(
                self.id.with("ClockModeTabViewer"),
                ClockMode::iter().collect::<Vec<_>>(),
                0,
            )
            .show(ui)
            .selected_tab;

            let clock_value: ClockValue = match clock_mode {
                ClockMode::Timecode => timing_handler.current_timecode_packet().clone().into(),
                ClockMode::LocalTime => ClockValue::local_time(),
                ClockMode::Utc => ClockValue::utc_time(),
            };

            egui::Frame::new()
                .fill(ecolor::Color32::BLACK)
                .show(ui, |ui| {
                    ui.add_sized(
                        ui.available_size(),
                        egui::Label::new(
                            egui::RichText::new(clock_value.to_string())
                                .extra_letter_spacing(2.0)
                                .color(ecolor::Color32::LIGHT_GREEN)
                                .background_color(ecolor::Color32::BLACK)
                                .monospace()
                                .size(80.0),
                        ),
                    );
                });
        });
    }
}
