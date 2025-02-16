use egui_probe::Probe;

use crate::{
    fixture::{
        handler::FixtureHandler, presets::PresetHandler, sequence::cue::CueIdx,
        updatables::UpdatableHandler,
    },
    parser::nodes::action::ConfigTypeActionData,
    ui::error::DemexUiError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemexEditWindow {
    EditSequence(u32),
    EditSequenceCue(u32, CueIdx),
    EditExecutor(u32),
    EditFader(u32),
    EditPreset(u32),

    Config(ConfigTypeActionData),
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
            Self::EditPreset(preset_id) => {
                format!("Preset {}", preset_id)
            }
            Self::Config(config_type) => format!("Config {:?}", config_type),
        }
    }

    pub fn window_ui(
        &self,
        ui: &mut egui::Ui,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::EditSequence(sequence_id) => {
                Probe::new(preset_handler.get_sequence_mut(*sequence_id)?).show(ui);
            }
            Self::EditSequenceCue(sequence_id, cue_idx) => {
                Probe::new(
                    preset_handler
                        .get_sequence_mut(*sequence_id)
                        .unwrap()
                        .find_cue_mut(*cue_idx)
                        .ok_or(DemexUiError::RuntimeError("cue not found".to_owned()))?,
                )
                .show(ui);
            }
            Self::EditExecutor(executor_id) => {
                Probe::new(
                    updatable_handler
                        .executor_mut(*executor_id)
                        .ok_or(DemexUiError::RuntimeError("executor not found".to_owned()))?,
                )
                .show(ui);
            }
            Self::EditFader(fader_id) => {
                Probe::new(updatable_handler.fader_mut(*fader_id)?).show(ui);
            }
            Self::EditPreset(preset_id) => {
                Probe::new(preset_handler.get_preset_mut(*preset_id)?).show(ui);
            }
            Self::Config(config_type) => match config_type {
                ConfigTypeActionData::Output => {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        "A restart is required for output changes to take effect!",
                    );

                    let patch = fixture_handler.patch_mut();

                    Probe::new(patch.output_configs_mut())
                        .with_header("Outputs")
                        .show(ui);
                }
            },
        };

        Ok(())
    }
}
