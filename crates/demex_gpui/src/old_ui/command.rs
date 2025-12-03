use crate::{lexer::Lexer, ui::dlog::dialog::DemexGlobalDialogEntry};

use super::context::DemexUiContext;

pub fn ui_command_input(ctx: &egui::Context, context: &mut DemexUiContext) {
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
                                .background_color(ecolor::Color32::BLACK)
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
                        .text_color(ecolor::Color32::YELLOW),
                )
                .labelled_by(command_label.id);

            if context.window_handler.is_empty() {
                if ui.input_mut(|writer| writer.consume_key(egui::Modifiers::NONE, egui::Key::Tab))
                {
                    context.should_focus_command_input = true;
                }

                if command_input_field.lost_focus() {
                    context.should_focus_command_input = false;
                }

                if context.should_focus_command_input {
                    command_input_field.request_focus();
                }

                if command_input_field
                    .ctx
                    .input_mut(|i| i.key_pressed(egui::Key::Space))
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

                if context.is_command_input_empty {
                    if command_input_field
                        .ctx
                        .input_mut(|i| i.key_pressed(egui::Key::Backspace))
                    {
                        context.command.pop();
                    }

                    if command_input_field
                        .ctx
                        .input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Backspace))
                        || command_input_field.ctx.input_mut(|i| {
                            i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::Backspace)
                        })
                    {
                        context.command.clear();
                    }
                }

                context.is_command_input_empty = context.command_input.is_empty();

                let command_empty = context.command.is_empty() && context.command_input.is_empty();
                if !command_empty
                    && command_input_field
                        .ctx
                        .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
                {
                    let mut lexer = Lexer::new(&context.command_input);
                    let tokens = lexer.tokenize();

                    if let Ok(tokens) = tokens {
                        context.command.extend(tokens);

                        context.command_input.clear();

                        if let Err(err) = context.enqueue_cmd() {
                            log::warn!("Failed to parse cmd: {}", err);
                            context.add_dialog_entry(DemexGlobalDialogEntry::error(&err));
                        }

                        context.command.clear();

                        context.should_focus_command_input = false;
                    }
                }
            }
        });
    });
}
