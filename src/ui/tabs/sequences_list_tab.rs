use itertools::Itertools;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let preset_handler = context.preset_handler.read();

    ui.vertical(|ui| {
        ui.heading("Sequences");
        for (id, seq) in preset_handler
            .sequences()
            .iter()
            .sorted_by_key(|(id, _)| *id)
        {
            ui.horizontal(|ui| {
                ui.label(id.to_string());
                ui.label(seq.name());
                ui.label(format!("{} cues", seq.cues().len()));
            });
        }
    });
}
