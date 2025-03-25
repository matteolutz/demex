use crate::ui::context::DemexUiContext;

pub struct PatchViewComponent<'a> {
    context: &'a mut DemexUiContext,
    id_source: egui::Id,
}

impl<'a> PatchViewComponent<'a> {
    pub fn new(context: &'a mut DemexUiContext) -> Self {
        Self {
            context,
            id_source: egui::Id::new("DemexPatchViewComponent"),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("TODO");
    }
}
