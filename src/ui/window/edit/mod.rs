use builder_cue::{edit_builder_cue_ui, DisplayEntry, PresetDisplayEntry};
use egui_probe::Probe;
use group::edit_group_ui;
use itertools::Itertools;
use preset::edit_preset_ui;

use crate::{
    fixture::{
        handler::FixtureHandler,
        patch::Patch,
        presets::{preset::FixturePresetId, PresetHandler},
        sequence::cue::CueIdx,
        updatables::UpdatableHandler,
    },
    parser::nodes::action::ConfigTypeActionData,
    ui::error::DemexUiError,
};

pub mod builder_cue;
pub mod group;
pub mod preset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemexEditWindow {
    EditSequence(u32),
    EditSequenceCue(u32, CueIdx),
    EditExecutor(u32),
    EditFader(u32),
    EditPreset(FixturePresetId),

    EditPreset2(FixturePresetId),

    EditGroup(u32),

    EditBuilderCue(u32, CueIdx),

    ConfigOverview,
    Config(ConfigTypeActionData),
}

impl DemexEditWindow {
    pub fn title(&self) -> String {
        match self {
            Self::EditSequence(sequence_id) => format!("Sequence {}", sequence_id),
            Self::EditSequenceCue(sequence_id, (cue_idx_major, cue_idx_minor))
            | Self::EditBuilderCue(sequence_id, (cue_idx_major, cue_idx_minor)) => {
                format!(
                    "Sequence {} - Cue {}.{}",
                    sequence_id, cue_idx_major, cue_idx_minor
                )
            }

            Self::EditExecutor(executor_id) => format!("Executor {}", executor_id),
            Self::EditFader(fader_id) => format!("Fader {}", fader_id),
            Self::EditPreset(preset_id) | Self::EditPreset2(preset_id) => {
                format!("Preset {}", preset_id)
            }
            Self::EditGroup(group_id) => format!("Group {}", group_id),
            Self::ConfigOverview => "Config".to_owned(),
            Self::Config(config_type) => format!("Config {:?}", config_type),
        }
    }

    pub fn should_fullscreen(&self) -> bool {
        match self {
            Self::EditBuilderCue(_, _) => true,
            Self::Config(_) => true,
            Self::EditPreset2(_) => true,
            Self::EditGroup(_) => true,
            _ => false,
        }
    }

    pub fn window_ui(
        &self,
        ui: &mut egui::Ui,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        patch: &mut Patch,
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
            Self::EditPreset2(preset_id) => {
                let preset = preset_handler
                    .get_preset_mut(*preset_id)
                    .map_err(|_| DemexUiError::RuntimeError("Preset not found".to_string()))?;

                edit_preset_ui(ui, preset);
            }
            Self::EditBuilderCue(sequence_id, cue_idx) => {
                let groups = preset_handler
                    .groups()
                    .iter()
                    .map(|(id, group)| DisplayEntry {
                        id: *id,
                        name: group.name().to_owned(),
                    })
                    .collect::<Vec<_>>();

                let presets = preset_handler
                    .presets()
                    .iter()
                    .map(|(id, preset)| PresetDisplayEntry {
                        id: *id,
                        name: format!("{} - {}", id.feature_group.name(), preset.name()),
                    })
                    .sorted_by_key(|e| e.id)
                    .collect::<Vec<_>>();

                let sequence = preset_handler.get_sequence_mut(*sequence_id)?;
                let cue = sequence
                    .find_cue_mut(*cue_idx)
                    .ok_or(DemexUiError::RuntimeError("cue not found".to_owned()))?;

                edit_builder_cue_ui(ui, *sequence_id, cue, groups, presets);
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
            Self::EditGroup(group_id) => {
                let group = preset_handler.get_group_mut(*group_id)?;

                edit_group_ui(ui, group, fixture_handler);
            }
            Self::ConfigOverview => {
                ui.heading("Config overview");
            }
            Self::Config(config_type) => match config_type {
                ConfigTypeActionData::Output => {
                    ui.colored_label(
                        ecolor::Color32::YELLOW,
                        "A restart is required for output changes to take effect!",
                    );

                    Probe::new(patch.output_configs_mut())
                        .with_header("Outputs")
                        .show(ui);
                }
                ConfigTypeActionData::Patch => {
                    ui.heading("Patch");
                }
            },
        };

        Ok(())
    }
}
