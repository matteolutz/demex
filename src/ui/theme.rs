#[derive(Debug, Clone)]
pub enum DemexUiTheme {
    Default,
    Catppuccin(catppuccin_egui::Theme),
}

impl DemexUiTheme {
    pub fn apply(self, ctx: &egui::Context) {
        match self {
            Self::Default => {}
            Self::Catppuccin(theme) => catppuccin_egui::set_theme(ctx, theme),
        }
    }
}
