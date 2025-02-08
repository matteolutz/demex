use egui_probe::Probe;

use crate::fixture::{
    channel::FixtureChannel, presets::PresetHandler, sequence::cue::CueIdx,
    updatables::UpdatableHandler,
};

#[derive(Debug, Clone)]
pub enum DemexEditWindow {
    EditSequence(u32),
    EditSequenceCue(u32, CueIdx),
    EditExecutor(u32),
    EditFader(u32),
    EditPreset(u16, u32),
}

impl DemexEditWindow {
    pub fn title(&self) -> String {
        match self {
            Self::EditSequence(sequence_id) => format!("Sequence {}", sequence_id),
            Self::EditSequenceCue(sequence_id, (cue_idx_major, cue_idx_minor)) => {
                format!(
                    "Sequence {} - Cue {}.{}",
                    sequence_id, cue_idx_major, cue_idx_minor
                )
            }
            Self::EditExecutor(executor_id) => format!("Executor {}", executor_id),
            Self::EditFader(fader_id) => format!("Fader {}", fader_id),
            Self::EditPreset(fixture_id, preset_id) => {
                format!(
                    "{} Preset {}",
                    FixtureChannel::name_by_id(*fixture_id),
                    preset_id
                )
            }
        }
    }

    pub fn ui(
        &self,
        ctx: &egui::Context,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> bool {
        egui::Window::new(self.title())
            .show(ctx, |ui| {
                if ui.input(|reader| reader.key_pressed(egui::Key::Escape)) {
                    return true;
                }

                match self {
                    Self::EditSequence(sequence_id) => {
                        Probe::new(preset_handler.get_sequence_mut(*sequence_id).unwrap()).show(ui);
                    }
                    Self::EditSequenceCue(sequence_id, cue_idx) => {
                        Probe::new(
                            preset_handler
                                .get_sequence_mut(*sequence_id)
                                .unwrap()
                                .find_cue_mut(*cue_idx)
                                .unwrap(),
                        )
                        .show(ui);
                    }
                    Self::EditExecutor(executor_id) => {
                        Probe::new(updatable_handler.executor_mut(*executor_id).unwrap()).show(ui);
                    }
                    Self::EditFader(fader_id) => {
                        Probe::new(updatable_handler.fader_mut(*fader_id).unwrap()).show(ui);
                    }
                    Self::EditPreset(channel_type, preset_id) => {
                        Probe::new(
                            preset_handler
                                .get_preset_mut(*preset_id, *channel_type)
                                .unwrap(),
                        )
                        .show(ui);
                    }
                }

                let close_button = ui.button("Close");
                if close_button.clicked() {
                    return true;
                }

                false
            })
            .map(|inner| inner.inner)
            .unwrap_or(None)
            .unwrap_or(false)
    }
}
