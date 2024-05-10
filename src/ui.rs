pub struct UIApp {
    command_input: String,
}

impl Default for UIApp {
    fn default() -> Self {
        Self {
            command_input: String::new(),
        }
    }
}

impl eframe::App for UIApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("demex");
            ui.horizontal(|ui| {
                let command_label = ui.label("Command");
                let command_input_field = ui
                    .text_edit_singleline(&mut self.command_input)
                    .labelled_by(command_label.id);

                if command_input_field.lost_focus()
                    && command_input_field
                        .ctx
                        .input(|i| i.key_pressed(eframe::egui::Key::Enter))
                {
                    println!("command: {}", self.command_input);
                }
            });
        });
    }
}
