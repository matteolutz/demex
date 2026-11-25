use demex_core::color::color_space::RgbColorSpace;
use gpui::SharedString;
use gpui_component::select::SelectItem;

#[derive(Debug, Clone)]
pub struct ColorSpaceSelectItem {
    pub color_space: RgbColorSpace,
    pub name: SharedString,
}

impl SelectItem for ColorSpaceSelectItem {
    type Value = RgbColorSpace;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.color_space
    }
}

impl From<RgbColorSpace> for ColorSpaceSelectItem {
    fn from(color_space: RgbColorSpace) -> Self {
        Self {
            color_space,
            name: color_space.to_string().into(),
        }
    }
}
