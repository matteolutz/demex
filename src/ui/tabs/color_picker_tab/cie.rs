use egui::epaint::CubicBezierShape;

use crate::{
    color::color_space::{RgbColorSpace, RgbValue},
    ui::context::DemexUiContext,
};

pub struct CieColorPickerComponent<'a> {
    color_space: RgbColorSpace,
    context: &'a DemexUiContext,
}

impl<'a> CieColorPickerComponent<'a> {
    pub fn new(context: &'a DemexUiContext, color_space: RgbColorSpace) -> Self {
        Self {
            context,
            color_space,
        }
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        selected_color: Option<egui::Color32>,
    ) -> Option<RgbValue> {
        let cie_uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.15), egui::pos2(0.75, 1.0));

        let available_rect = ui.available_rect_before_wrap();

        let square_size = available_rect.width().min(available_rect.height());
        let painter_rect = egui::Rect::from_min_size(
            available_rect.min + egui::vec2(0.0, square_size),
            egui::vec2(square_size, square_size),
        );

        let (response, painter) =
            ui.allocate_painter(painter_rect.size(), egui::Sense::click_and_drag());

        let texture_id = self.context.texture_handles[0].id();
        painter.image(texture_id, response.rect, cie_uv, egui::Color32::WHITE);

        let to_cie_transform = emath::RectTransform::from_to(
            response.rect,
            // Flip Y axis to match CIE 1931 color space
            cie_uv
                .with_min_y(1.0 - cie_uv.min.y)
                .with_max_y(1.0 - cie_uv.max.y),
        );
        let to_screen_transform = to_cie_transform.inverse();

        /*
        painter.rect_stroke(
            response.rect,
            0.0,
            (2.0, egui::Color32::WHITE),
            egui::StrokeKind::Middle,
        );
        */

        // Planckian locus
        painter.add(CubicBezierShape::from_points_stroke(
            [
                to_screen_transform.transform_pos(egui::pos2(0.274, 0.2727)),
                to_screen_transform.transform_pos(egui::pos2(0.316, 0.3230)),
                to_screen_transform.transform_pos(egui::pos2(0.358, 0.3613)),
                to_screen_transform.transform_pos(egui::pos2(0.400, 0.3855)),
            ],
            false,
            egui::Color32::TRANSPARENT,
            (1.0, egui::Color32::BLACK),
        ));
        painter.add(CubicBezierShape::from_points_stroke(
            [
                to_screen_transform.transform_pos(egui::pos2(0.400, 0.3855)),
                to_screen_transform.transform_pos(egui::pos2(0.420, 0.3987)),
                to_screen_transform.transform_pos(egui::pos2(0.440, 0.4064)),
                to_screen_transform.transform_pos(egui::pos2(0.460, 0.4108)),
            ],
            false,
            egui::Color32::TRANSPARENT,
            (1.0, egui::Color32::BLACK),
        ));
        painter.add(CubicBezierShape::from_points_stroke(
            [
                to_screen_transform.transform_pos(egui::pos2(0.460, 0.4108)),
                to_screen_transform.transform_pos(egui::pos2(0.487, 0.4208)),
                to_screen_transform.transform_pos(egui::pos2(0.513, 0.4254)),
                to_screen_transform.transform_pos(egui::pos2(0.540, 0.4282)),
            ],
            false,
            egui::Color32::TRANSPARENT,
            (1.0, egui::Color32::BLACK),
        ));

        let color_space_vertices = self
            .color_space
            .gammut()
            .into_iter()
            .map(|color| color.to_xy())
            .map(|(x, y)| to_screen_transform.transform_pos_clamped(egui::pos2(x, y)))
            .collect::<Vec<_>>();

        painter.add(egui::epaint::PathShape::convex_polygon(
            color_space_vertices,
            egui::Color32::TRANSPARENT,
            (2.0, egui::Color32::WHITE),
        ));

        let is_interacted_with = response.clicked() || response.dragged();

        let display_color = if is_interacted_with {
            let interact_pos = response.interact_pointer_pos().unwrap();
            let norm_pos = to_cie_transform.transform_pos_clamped(interact_pos);

            let color = RgbValue::from_xy_bri(norm_pos.x, norm_pos.y, 1.0, self.color_space);
            Some(color)
        } else {
            selected_color.map(|color| RgbValue::from_color(color, self.color_space))
        };

        if let Some(color) = display_color {
            let (x, y) = color.to_xy();
            painter.circle_filled(
                to_screen_transform.transform_pos_clamped(egui::pos2(x, y)),
                5.0,
                color.invert(),
            );
        }

        if is_interacted_with {
            Some(display_color.unwrap_or_else(|| RgbValue::new(1.0, 1.0, 1.0, self.color_space)))
        } else {
            None
        }
    }
}
