pub mod cie;
pub mod gels;
pub mod sliders;

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum ColorPickerTabType {
    Cie,
    Rgb,
    Sliders,
    Gels,
}

impl std::fmt::Display for ColorPickerTabType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorPickerTabType::Cie => write!(f, "CIE"),
            ColorPickerTabType::Rgb => write!(f, "RGB"),
            ColorPickerTabType::Sliders => write!(f, "Sliders"),
            ColorPickerTabType::Gels => write!(f, "Gels"),
        }
    }
}

use strum::{EnumIter, IntoEnumIterator};

use crate::{
    color::color_space::RgbColorSpace,
    ui::{components::tab_viewer::TabViewer, context::DemexUiContext},
};

#[derive(Default, Clone)]
pub struct ColorPickerState {
    cie_color_space: RgbColorSpace,
}

pub struct ColorPickerComponent<'a> {
    context: &'a mut DemexUiContext,
    id_source: egui::Id,
}

impl<'a> ColorPickerComponent<'a> {
    pub fn new(context: &'a mut DemexUiContext) -> Self {
        let id_source = egui::Id::new("DemexColorPickerComponent");
        Self { context, id_source }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let mut state: ColorPickerState = ui
            .data(|reader| reader.get_temp(self.id_source))
            .unwrap_or_default();

        let mut initial_color = {
            let fixture_handler = self.context.fixture_handler.read();
            let preset_handler = self.context.preset_handler.read();
            let timing_handler = self.context.timing_handler.read();
            let patch = self.context.patch.read();

            let fixtures = self
                .context
                .global_fixture_select
                .as_ref()
                .map(|fixture_selection| fixture_selection.fixtures());

            if let Some(fixtures) = fixtures {
                fixtures
                    .iter()
                    .map(|&fixture_id| fixture_handler.fixture_immut(fixture_id).unwrap())
                    .find_map(|fixture| {
                        fixture
                            .rgb_color(patch.fixture_types(), &preset_handler, &timing_handler)
                            .ok()
                    })
                    .map(|[r, g, b]| {
                        egui::Color32::from_rgb(
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8,
                        )
                    })
            } else {
                None
            }
        }
        .unwrap_or(egui::Color32::TRANSPARENT);

        let update_color: Option<egui::Color32> = ui
            .vertical(|ui| {
                let (response, painter) = ui
                    .allocate_painter(egui::vec2(ui.available_width(), 10.0), egui::Sense::empty());
                painter.rect_filled(response.rect, 0, initial_color);

                let selected_color_picker_type = TabViewer::new(
                    self.id_source.with("ColorPickerTabType"),
                    ColorPickerTabType::iter().collect::<Vec<_>>(),
                    0,
                )
                .show(ui);

                match selected_color_picker_type.selected_tab {
                    ColorPickerTabType::Cie => {
                        ui.vertical(|ui| {
                            egui::ComboBox::new(
                                self.id_source.with("CieColorSpace"),
                                "Color Space",
                            )
                            .selected_text(state.cie_color_space.to_string())
                            .show_ui(ui, |ui| {
                                for color_space in RgbColorSpace::iter() {
                                    ui.selectable_value(
                                        &mut state.cie_color_space,
                                        color_space,
                                        color_space.to_string(),
                                    );
                                }
                            });

                            let updated_color = cie::CieColorPickerComponent::new(
                                self.context,
                                state.cie_color_space,
                            )
                            .show(ui, Some(initial_color));

                            updated_color.map(|color| color.into())
                        })
                        .inner
                    }
                    ColorPickerTabType::Rgb => {
                        if egui::color_picker::color_picker_color32(
                            ui,
                            &mut initial_color,
                            egui::color_picker::Alpha::Opaque,
                        ) {
                            Some(initial_color)
                        } else {
                            None
                        }
                    }
                    ColorPickerTabType::Sliders => {
                        sliders::SlidersColorPickerComponent::new().show(ui, initial_color)
                    }
                    ColorPickerTabType::Gels => {
                        gels::GelPickerComponent::new(self.id_source.with("Gels")).show(ui)
                    }
                }
            })
            .inner;

        if let (Some(update_color), Some(global_fixture_select)) =
            (update_color, self.context.global_fixture_select.as_ref())
        {
            let mut fixture_handler = self.context.fixture_handler.write();
            let patch = self.context.patch.read();

            for fixture_id in global_fixture_select.fixtures() {
                let fixture = fixture_handler.fixture(*fixture_id);
                if fixture.is_none() {
                    continue;
                }

                if let Err(err) = fixture.unwrap().apply_rgb_color(
                    patch.fixture_types(),
                    [
                        update_color.r() as f32 / 255.0,
                        update_color.g() as f32 / 255.0,
                        update_color.b() as f32 / 255.0,
                    ],
                ) {
                    log::warn!(
                        "Failed to update fixture (id: {}) color: {}",
                        fixture_id,
                        err
                    );
                }
            }
        }

        ui.data_mut(|writer| writer.insert_temp(self.id_source, state));
    }
}
