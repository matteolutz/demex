pub const DRAFT_ICON: egui::ImageSource =
    egui::include_image!("../../../../assets/icons/draft.png");
pub const SETTINGS_ICON: egui::ImageSource =
    egui::include_image!("../../../../assets/icons/settings.png");
pub const DELETE_ICON: egui::ImageSource =
    egui::include_image!("../../../../assets/icons/delete.png");
pub const CLOSE_ICON: egui::ImageSource =
    egui::include_image!("../../../../assets/icons/close.png");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemexIcon {
    Draft,
    Settings,
    Delete,
    Close,
}

impl DemexIcon {
    pub fn image_source(&self) -> egui::ImageSource {
        match self {
            Self::Draft => DRAFT_ICON,
            Self::Settings => SETTINGS_ICON,
            Self::Delete => DELETE_ICON,
            Self::Close => CLOSE_ICON,
        }
    }

    pub fn image(&self, size: egui::Vec2) -> egui::Image {
        egui::Image::new(self.image_source()).fit_to_exact_size(size)
    }

    pub fn button_image(&self) -> egui::Image {
        self.image(egui::vec2(20.0, 20.0))
    }
}
