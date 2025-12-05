use gpui::App;
use gpui_component::highlighter::{LanguageConfig, LanguageRegistry};

pub mod assets;
pub mod components;
pub mod config;
pub mod ext;
pub mod pane;
pub mod panels;
pub mod titlebar;
pub mod utils;
pub mod window;
pub mod wm;

pub fn init(cx: &mut App) -> gpui::Result<()> {
    register_demex_language();

    assets::init(cx)?;
    panels::init(cx);
    window::init(cx);

    Ok(())
}

fn register_demex_language() {
    let lang_config = LanguageConfig::new(
        "demex",
        tree_sitter_demex::LANGUAGE.into(),
        vec![],
        tree_sitter_demex::HIGHLIGHTS_QUERY,
        "",
        "",
    );

    LanguageRegistry::singleton().register("demex", &lang_config);
}
