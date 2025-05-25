use crate::ui::components::separator::padded_separator;

pub struct SlidersColorPickerComponent {}

impl SlidersColorPickerComponent {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&self, ui: &mut egui::Ui, initial_color: egui::Color32) -> Option<egui::Color32> {
        let mut picker_color = initial_color;

        // R
        ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
            if let Some(val) = val {
                picker_color = egui::Color32::from_rgb(
                    (val * 255.0) as u8,
                    picker_color.g(),
                    picker_color.b(),
                );
                val
            } else {
                picker_color.r() as f64 / 255.0
            }
        }));

        // G
        ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
            if let Some(val) = val {
                picker_color = egui::Color32::from_rgb(
                    picker_color.r(),
                    (val * 255.0) as u8,
                    picker_color.b(),
                );
                val
            } else {
                picker_color.g() as f64 / 255.0
            }
        }));

        // B
        ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
            if let Some(val) = val {
                picker_color = egui::Color32::from_rgb(
                    picker_color.r(),
                    picker_color.g(),
                    (val * 255.0) as u8,
                );
                val
            } else {
                picker_color.b() as f64 / 255.0
            }
        }));

        padded_separator(ui);

        let cmyk_k = 0.0;

        // C
        ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
            if let Some(val) = val {
                let r = 255.0 * (1.0 - val) * (1.0 - cmyk_k);
                picker_color = egui::Color32::from_rgb(r as u8, picker_color.g(), picker_color.b());
                val
            } else {
                (1.0 - picker_color.r() as f64 / 255.0 - cmyk_k) / (1.0 - cmyk_k)
            }
        }));

        // M
        ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
            if let Some(val) = val {
                let g = 255.0 * (1.0 - val) * (1.0 - cmyk_k);
                picker_color = egui::Color32::from_rgb(picker_color.r(), g as u8, picker_color.b());
                val
            } else {
                (1.0 - picker_color.g() as f64 / 255.0 - cmyk_k) / (1.0 - cmyk_k)
            }
        }));

        // Y
        ui.add(egui::Slider::from_get_set(0.0..=1.0, |val| {
            if let Some(val) = val {
                let b = 255.0 * (1.0 - val) * (1.0 - cmyk_k);
                picker_color = egui::Color32::from_rgb(picker_color.r(), picker_color.g(), b as u8);
                val
            } else {
                (1.0 - picker_color.b() as f64 / 255.0 - cmyk_k) / (1.0 - cmyk_k)
            }
        }));

        (picker_color != initial_color).then_some(picker_color)
    }
}
