#[derive(Debug)]
pub struct LayoutProjection {
    zoom: f32,
    center: egui::Vec2,
}

impl Default for LayoutProjection {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            center: egui::Vec2::default(),
        }
    }
}

impl LayoutProjection {
    pub fn with_zoom(zoom: f32) -> Self {
        Self {
            zoom,
            center: egui::Vec2::default(),
        }
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn center(&self) -> &egui::Vec2 {
        &self.center
    }

    pub fn zoom_mut(&mut self) -> &mut f32 {
        &mut self.zoom
    }

    pub fn center_mut(&mut self) -> &mut egui::Vec2 {
        &mut self.center
    }
}

impl LayoutProjection {
    pub fn project(&self, world_pos: &egui::Pos2, screen: &egui::Rect) -> egui::Pos2 {
        let world_pos_vec = world_pos.to_vec2() * self.zoom;
        let screen_center = screen.min.to_vec2() + (screen.size() / 2.0);
        let screen_pos = (self.center * self.zoom) + world_pos_vec + screen_center;
        screen_pos.to_pos2()
    }

    pub fn unproject(&self, screen_pos: &egui::Pos2, screen: &egui::Rect) -> egui::Pos2 {
        let world_pos_vec = screen_pos.to_vec2();
        let screen_center = screen.min.to_vec2() + (screen.size() / 2.0);
        let mut offset = world_pos_vec - screen_center;

        offset /= self.zoom;

        offset.to_pos2()
    }

    pub fn scale(&self, vec: &egui::Vec2) -> egui::Vec2 {
        *vec * self.zoom
    }

    pub fn scale_ref(&self, vec: &mut egui::Vec2) {
        vec.x *= self.zoom;
        vec.y *= self.zoom;
    }
}

#[cfg(test)]
mod tests {
    use egui::Rect;

    use super::LayoutProjection;

    #[test]
    pub fn test_projection_basic() {
        let projection = LayoutProjection::with_zoom(2.0);
        let screen = Rect::from_min_max(egui::pos2(10.0, 10.0), egui::pos2(20.0, 20.0));

        let orig_point = egui::pos2(-1.0, 0.0);

        let screen_point = projection.project(&orig_point, &screen);
        assert_eq!(screen_point, egui::pos2(13.0, 15.0));

        let unprojected_point = projection.unproject(&screen_point, &screen);
        assert_eq!(unprojected_point, orig_point);
    }
}
