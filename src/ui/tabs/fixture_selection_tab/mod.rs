use crate::lexer::token::Token;

const _ELEMENT_SIZE: f32 = 40.0;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    if let Some(fixture_selection) = context.global_fixture_select.as_ref() {
        ui.vertical(|ui| {
            if ui.button("Sel").clicked() {
                context
                    .command
                    .extend_from_slice(&[Token::KeywordFixturesSelected]);
            }

            ui.horizontal(|ui| {
                for offset_idx in 0..fixture_selection.num_offsets() {
                    ui.vertical(|ui| {
                        for fixture_id in fixture_selection.fixtures_with_offset_idx(offset_idx) {
                            let _ = ui.button(format!("{}", fixture_id));
                        }
                    });
                }
            });
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("No fixtures selected");
        });
    }
}
