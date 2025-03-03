use crate::{lexer::Lexer, ui::dlog::dialog::DemexGlobalDialogEntry};

use super::context::DemexUiContext;

pub fn ui_command_input(ctx: &egui::Context, context: &mut DemexUiContext, cmd_af: bool) {
    eframe::egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.add_space(10.0);

        let command_label = ui.label("Command");

        let command_font = eframe::egui::FontId::new(16.0, eframe::egui::FontFamily::Monospace);

        ui.horizontal(|ui| {
            if !context.command.is_empty() {
                ui.horizontal(|ui| {
                    for token in &context.command {
                        ui.label(
                            eframe::egui::RichText::from(token.to_string())
                                .background_color(eframe::egui::Color32::BLACK)
                                .color(token)
                                .font(command_font.clone()),
                        );
                    }
                });
            }

            let command_input_field = ui
                .add_sized(
                    ui.available_size(),
                    eframe::egui::TextEdit::singleline(&mut context.command_input)
                        .font(command_font)
                        .text_color(eframe::egui::Color32::YELLOW),
                )
                .labelled_by(command_label.id);

            if context.window_handler.is_empty() {
                if command_input_field
                    .ctx
                    .input(|i| i.key_pressed(eframe::egui::Key::Space))
                {
                    let mut lexer = Lexer::new(&context.command_input);
                    let tokens = lexer.tokenize();

                    if let Ok(tokens) = tokens {
                        context
                            .command
                            .extend(tokens.iter().take(tokens.len() - 1).cloned());

                        context.command_input.clear();
                    }
                }

                if context.is_command_input_empty
                    && command_input_field
                        .ctx
                        .input(|i| i.key_pressed(eframe::egui::Key::Backspace))
                {
                    if command_input_field
                        .ctx
                        .input(|i| i.modifiers.ctrl || i.modifiers.mac_cmd)
                    {
                        context.command.clear();
                    } else {
                        context.command.pop();
                    }
                }

                context.is_command_input_empty = context.command_input.is_empty();

                if !command_input_field.has_focus()
                    && (cmd_af
                        || ui.input_mut(|reader| {
                            reader.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                        }))
                {
                    command_input_field.request_focus();
                }

                if command_input_field
                    .ctx
                    .input(|i| i.key_pressed(eframe::egui::Key::Enter))
                {
                    let mut lexer = Lexer::new(&context.command_input);
                    let tokens = lexer.tokenize();

                    if let Ok(tokens) = tokens {
                        context.command.extend(tokens);

                        context.command_input.clear();

                        if let Err(e) = context.run_cmd() {
                            log::warn!("{}", e);
                            context.add_dialog_entry(DemexGlobalDialogEntry::error(e.as_ref()));
                        }

                        context.command.clear();
                    }
                }
            }
        });
    });
}
