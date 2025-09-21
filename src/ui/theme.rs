#[derive(Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum DemexUiThemeAttribute {
    Default,
    CatpuccinFrappe,
    CatpuccinMacchiato,
    CatpuccinMocha,
    CatpuccinLatte,

    #[cfg(feature = "reui")]
    ReUi,
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

            #[cfg(feature = "reui")]
            DemexUiThemeAttribute::ReUi => Self::ReUi,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DemexUiTheme {
    Default,
    Catppuccin(catppuccin_egui::Theme),

    #[cfg(feature = "reui")]
    ReUi,
}

impl DemexUiTheme {
    pub fn native_options(&self, viewport: eframe::egui::ViewportBuilder) -> eframe::NativeOptions {
        match self {
            Self::Default | Self::Catppuccin(_) => eframe::NativeOptions {
                viewport,
                ..Default::default()
            },

            #[cfg(feature = "reui")]
            Self::ReUi => eframe::NativeOptions {
                viewport: viewport
                    .with_decorations(!re_ui::CUSTOM_WINDOW_DECORATIONS)
                    .with_titlebar_buttons_shown(!re_ui::CUSTOM_WINDOW_DECORATIONS)
                    .with_transparent(re_ui::CUSTOM_WINDOW_DECORATIONS),
                ..Default::default()
            },
        }
    }

    pub fn apply(self, ctx: &egui::Context) {
        match self {
            Self::Default => {}
            Self::Catppuccin(theme) => catppuccin_egui::set_theme(ctx, theme),

            #[cfg(feature = "reui")]
            Self::ReUi => {
                re_ui::apply_style_and_install_loaders(ctx);
            }
        }
    }
}
