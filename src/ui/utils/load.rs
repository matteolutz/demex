use std::sync::Arc;

fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels: image::FlatSamples<&[u8]> = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

pub fn load_textures(ctx: &egui::Context) -> Vec<egui::TextureHandle> {
    let cie_texture = ctx.load_texture(
        "CIE",
        load_image_from_memory(include_bytes!("../../../assets/cie/CIE_1931_Chromaticity_Diagram_CIE_1931_2_Degree_Standard_Observer.png")).unwrap(),
        egui::TextureOptions::default()
    );

    vec![cie_texture]
}

pub fn load_fonts() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "open-sans".to_string(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/OpenSans-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "jetbrains-mono".to_string(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/JetBrainsMono-Regular.ttf"
        ))),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "open-sans".to_string());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "jetbrains-mono".to_string());

    fonts.families.insert(
        egui::FontFamily::Name("Timecode".into()),
        vec!["timecode".to_string()],
    );

    fonts
}
