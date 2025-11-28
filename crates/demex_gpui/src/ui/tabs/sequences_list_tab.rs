use itertools::Itertools;

use crate::lexer::token::Token;

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
                let name_button = ui.button(seq.name());
                if name_button.clicked() {
                    context
                        .command
                        .extend_from_slice(&[Token::KeywordSequence, Token::Integer(*id)]);
                }

                ui.label(format!("{} cues", seq.cues().len()));

                ui.label("(");
                for (cue_idx_major, cue_idx_minor) in seq.cues().iter().map(|c| c.cue_idx()) {
                    let cue_button = ui.button(format!("{}.{}", cue_idx_major, cue_idx_minor));
                    if cue_button.clicked() {
                        context.command.extend_from_slice(&[
                            Token::KeywordSequence,
                            Token::Integer(*id),
                            Token::KeywordCue,
                            Token::FloatingPoint(0.0, (cue_idx_major, cue_idx_minor)),
                        ]);
                    }
                }

                if ui.button("+").clicked() {
                    context.command.extend_from_slice(&[
                        Token::KeywordRecord,
                        Token::KeywordSequence,
                        Token::Integer(*id),
                        Token::KeywordCue,
                        Token::KeywordNext,
                    ]);
                }

                ui.label(")");
            });
        }

        if ui.button("+").clicked() {
            context.command.extend_from_slice(&[
                Token::KeywordCreate,
                Token::KeywordSequence,
                Token::KeywordNext,
            ]);
        }
    });
}
