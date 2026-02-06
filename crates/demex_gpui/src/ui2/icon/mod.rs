use gpui_component::IconNamed;

pub enum DemexIconName {
    Home,
    CallMade,
}

impl IconNamed for DemexIconName {
    fn path(self) -> gpui::SharedString {
        match self {
            Self::Home => "icons/home.svg".into(),
            Self::CallMade => "icons/call-made.svg".into(),
        }
    }
}
