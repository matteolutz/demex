use gpui::App;

pub fn init(cx: &mut App) -> gpui::Result<()> {
    load_fonts(cx)?;
    Ok(())
}

use anyhow::anyhow;
use gpui::*;
use rust_embed::RustEmbed;
use std::borrow::Cow;

/// An asset source that loads assets from the `./assets` folder.
#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "icons/**/*.svg"]
#[include = "fonts/**/*.ttf"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

fn load_fonts(cx: &mut App) -> gpui::Result<()> {
    let font_paths = cx.asset_source().list("fonts")?;

    let embedded_fonts = font_paths
        .into_iter()
        .filter(|p| p.ends_with(".ttf"))
        .map(|p| cx.asset_source().load(&p))
        .filter_map(|res| res.ok().flatten())
        .collect::<Vec<_>>();

    log::debug!("Loaded {} fonts", embedded_fonts.len());
    cx.text_system().add_fonts(embedded_fonts)?;

    Ok(())
}
