#[derive(Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum DemexUiThemeAttribute {
    Default,
    CatpuccinFrappe,
    CatpuccinMacchiato,
    CatpuccinMocha,
    CatpuccinLatte,
}

impl From<DemexUiThemeAttribute> for DemexUiTheme {
    fn from(attr: DemexUiThemeAttribute) -> Self {
        match attr {
            DemexUiThemeAttribute::Default => Self::Default,
            DemexUiThemeAttribute::CatpuccinFrappe => Self::Catppuccin(catppuccin_egui::FRAPPE),
            DemexUiThemeAttribute::CatpuccinMacchiato => {
                Self::Catppuccin(catppuccin_egui::MACCHIATO)
            }
            DemexUiThemeAttribute::CatpuccinMocha => Self::Catppuccin(catppuccin_egui::MOCHA),
            DemexUiThemeAttribute::CatpuccinLatte => Self::Catppuccin(catppuccin_egui::LATTE),
        }
    }
}

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
