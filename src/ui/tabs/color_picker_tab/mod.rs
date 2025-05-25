pub mod cie;

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
    color::{color_space::RgbColorSpace, gel::ColorGelType},
    ui::{
        components::{separator::padded_separator, tab_viewer::TabViewer},
        context::DemexUiContext,
        utils::{color::color_to_luma, painter::painter_layout_centered},
    },
};

#[derive(Default, Clone)]
pub struct ColorPickerState {
    cie_color_space: RgbColorSpace,
    debug_color: egui::Color32,
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

        ui.vertical(|ui| {
            let (response, painter) =
                ui.allocate_painter(egui::vec2(ui.available_width(), 10.0), egui::Sense::empty());
            painter.rect_filled(response.rect, 0, state.debug_color);

            let selected_color_picker_type = TabViewer::new(
                self.id_source.with("ColorPickerTabType"),
                ColorPickerTabType::iter().collect::<Vec<_>>(),
                0,
            )
            .show(ui);

            match selected_color_picker_type.selected_tab {
                ColorPickerTabType::Cie => {
                    ui.vertical(|ui| {
                        egui::ComboBox::new(self.id_source.with("CieColorSpace"), "Color Space")
                            .selected_text(format!("{:?}", state.cie_color_space))
                            .show_ui(ui, |ui| {
                                for color_space in RgbColorSpace::iter() {
                                    ui.selectable_value(
                                        &mut state.cie_color_space,
                                        color_space,
                                        format!("{:?}", color_space),
                                    );
                                }
                            });

                        let updated_color =
                            cie::CieColorPickerComponent::new(self.context, state.cie_color_space)
                                .show(ui, Some(state.debug_color));

                        if let Some(updated_color) = updated_color {
                            state.debug_color = updated_color.into()
                        }
                    });
                }
                ColorPickerTabType::Rgb => {
                    egui::color_picker::color_picker_color32(
                        ui,
                        &mut state.debug_color,
                        egui::color_picker::Alpha::Opaque,
                    );
                }
                ColorPickerTabType::Sliders => {
                    // R
                    ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
                        if let Some(val) = val {
                            state.debug_color = egui::Color32::from_rgb(
                                (val * 255.0) as u8,
                                state.debug_color.g(),
                                state.debug_color.b(),
                            );
                            val
                        } else {
                            state.debug_color.r() as f64 / 255.0
                        }
                    }));

                    // G
                    ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
                        if let Some(val) = val {
                            state.debug_color = egui::Color32::from_rgb(
                                state.debug_color.r(),
                                (val * 255.0) as u8,
                                state.debug_color.b(),
                            );
                            val
                        } else {
                            state.debug_color.g() as f64 / 255.0
                        }
                    }));

                    // B
                    ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
                        if let Some(val) = val {
                            state.debug_color = egui::Color32::from_rgb(
                                state.debug_color.r(),
                                state.debug_color.g(),
                                (val * 255.0) as u8,
                            );
                            val
                        } else {
                            state.debug_color.b() as f64 / 255.0
                        }
                    }));

                    padded_separator(ui);

                    let cmyk_k = 0.0;

                    // C
                    ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
                        if let Some(val) = val {
                            let r = 255.0 * (1.0 - val) * (1.0 - cmyk_k);
                            state.debug_color = egui::Color32::from_rgb(
                                r as u8,
                                state.debug_color.g(),
                                state.debug_color.b(),
                            );
                            val
                        } else {
                            (1.0 - state.debug_color.r() as f64 / 255.0 - cmyk_k) / (1.0 - cmyk_k)
                        }
                    }));

                    // M
                    ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
                        if let Some(val) = val {
                            let g = 255.0 * (1.0 - val) * (1.0 - cmyk_k);
                            state.debug_color = egui::Color32::from_rgb(
                                state.debug_color.r(),
                                g as u8,
                                state.debug_color.b(),
                            );
                            val
                        } else {
                            (1.0 - state.debug_color.g() as f64 / 255.0 - cmyk_k) / (1.0 - cmyk_k)
                        }
                    }));

                    // Y
                    ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
                        if let Some(val) = val {
                            let b = 255.0 * (1.0 - val) * (1.0 - cmyk_k);
                            state.debug_color = egui::Color32::from_rgb(
                                state.debug_color.r(),
                                state.debug_color.g(),
                                b as u8,
                            );
                            val
                        } else {
                            (1.0 - state.debug_color.b() as f64 / 255.0 - cmyk_k) / (1.0 - cmyk_k)
                        }
                    }));
                }
                ColorPickerTabType::Gels => {
                    let selected_gel_type_viewer = TabViewer::new(
                        self.id_source.with("SelectedGelType"),
                        ColorGelType::iter().collect::<Vec<_>>(),
                        0,
                    )
                    .show(ui);

                    ui.add_space(10.0);

                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            ui.add_space(10.0);

                            for gel in selected_gel_type_viewer.selected_tab.gels() {
                                let (response, painter) = ui.allocate_painter(
                                    egui::vec2(ui.available_width(), 50.0),
                                    egui::Sense::click(),
                                );

                                let button_rect = response.rect.shrink2(egui::vec2(10.0, 2.5));

                                painter.rect_filled(button_rect, 5.0, gel.ecolor());
                                painter_layout_centered(
                                    &painter,
                                    gel.name().to_owned(),
                                    egui::FontId::proportional(12.0),
                                    if color_to_luma(&gel.ecolor()) > 0.5 {
                                        egui::Color32::BLACK
                                    } else {
                                        egui::Color32::WHITE
                                    },
                                    button_rect,
                                );

                                if response.hovered() {
                                    ui.output_mut(|o| {
                                        o.cursor_icon = egui::CursorIcon::PointingHand;
                                    });

                                    painter.rect_stroke(
                                        button_rect,
                                        5.0,
                                        (1.0, egui::Color32::WHITE),
                                        egui::StrokeKind::Middle,
                                    );
                                }

                                if response.clicked() {
                                    state.debug_color = gel.ecolor();
                                }
                            }

                            ui.add_space(10.0);
                        });
                }
            }
        });

        ui.data_mut(|writer| writer.insert_temp(self.id_source, state));
    }
}
