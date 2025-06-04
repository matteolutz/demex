use std::{path::PathBuf, sync::Arc};

use itertools::Itertools;
use parking_lot::RwLock;

use crate::{
    fixture::{
        handler::FixtureHandler,
        patch::{Patch, SerializablePatch},
        presets::PresetHandler,
        selection::FixtureSelection,
        timing::TimingHandler,
        updatables::UpdatableHandler,
    },
    headless::id::DemexProtoDeviceId,
    input::{device::DemexInputDeviceConfig, DemexInputDeviceHandler},
    lexer::token::Token,
    parser::{
        error::ParseError,
        nodes::{
            action::{queue::ActionQueue, result::ActionRunResult, Action, DeferredAction},
            fixture_selector::FixtureSelectorContext,
        },
        Parser2,
    },
    show::{context::ShowContext, ui::DemexShowUiConfig, DemexShow},
    ui::{
        edit_request::{UiEditRequest, UiEditRequestTrait},
        error::DemexUiError,
        tabs::DemexTab,
    },
    utils::{thread::DemexThreadStatsHandler, version::VERSION_STR},
};

use super::{
    dlog::{dialog::DemexGlobalDialogEntry, DemexLogEntry, DemexLogEntryType},
    tabs::encoders_tab::EncodersTabState,
    window::{DemexWindow, DemexWindowHandler},
};

pub type SaveShowFn =
    fn(DemexShow, Option<&PathBuf>) -> Result<PathBuf, Box<dyn std::error::Error>>;

pub struct DemexUiContext {
    pub command_input: String,
    pub is_command_input_empty: bool,
    pub should_focus_command_input: bool,

    pub ui_locked: bool,

    pub fixture_handler: Arc<RwLock<FixtureHandler>>,
    pub preset_handler: Arc<RwLock<PresetHandler>>,
    pub updatable_handler: Arc<RwLock<UpdatableHandler>>,
    pub timing_handler: Arc<RwLock<TimingHandler>>,
    pub patch: Arc<RwLock<Patch>>,

    pub texture_handles: Vec<egui::TextureHandle>,

    pub global_fixture_select: Option<FixtureSelection>,

    pub global_sequence_select: UiEditRequest<u32>,

    pub command: Vec<Token>,

    pub stats: Arc<RwLock<DemexThreadStatsHandler>>,

    pub logs: Vec<DemexLogEntry>,

    pub action_queue: ActionQueue,

    pub show_file: Option<PathBuf>,

    pub input_device_handler: DemexInputDeviceHandler,

    pub window_handler: DemexWindowHandler,

    pub ui_config: DemexShowUiConfig,

    pub encoders_tab_state: EncodersTabState,
}

impl DemexUiContext {
    pub fn add_dialog_entry(&mut self, dialog_entry: DemexGlobalDialogEntry) {
        self.window_handler.add_dialog_window(dialog_entry.clone());

        self.logs
            .push(DemexLogEntry::new(DemexLogEntryType::DialogEntry(
                dialog_entry,
            )));
    }

    pub fn execute_action_queue(&mut self, ui_config: &DemexShowUiConfig) {
        while !self.action_queue.is_empty() {
            let action = self.action_queue.dequeue().unwrap();

            if let Err(err) = self.run_and_handle_action(action, ui_config) {
                log::warn!("Failed to execute action: {}", err);
                self.add_dialog_entry(DemexGlobalDialogEntry::error(err.as_ref()));
            }
        }
    }

    pub fn enqueue_cmd(&mut self) -> Result<(), ParseError> {
        self.logs
            .push(DemexLogEntry::new(DemexLogEntryType::CommandEntry(
                self.command.clone(),
            )));

        let mut p = Parser2::new(&self.command);

        let action = p.parse().inspect_err(|err| {
            self.logs
                .push(DemexLogEntry::new(DemexLogEntryType::CommandFailedEntry(
                    err.to_string(),
                )))
        })?;

        self.action_queue.enqueue_now(action);

        Ok(())
    }

    pub fn ui_edit_requests(&mut self) -> [(&mut dyn UiEditRequestTrait, DemexTab); 1] {
        [(&mut self.global_sequence_select, DemexTab::SequenceEditor)]
    }

    pub fn load_show(
        show_context: &ShowContext,
        input_device_configs: Vec<DemexInputDeviceConfig>,
        ui_config: DemexShowUiConfig,
        show_file: Option<PathBuf>,
        stats: Arc<RwLock<DemexThreadStatsHandler>>,
        texture_handles: Vec<egui::TextureHandle>,
    ) -> Self {
        let patch = show_context.patch.clone();

        let fixture_handler = show_context.fixture_handler.clone();

        let preset_handler = show_context.preset_handler.clone();
        let updatable_handler = show_context.updatable_handler.clone();
        let timing_handler = show_context.timing_handler.clone();

        let input_device_handler = DemexInputDeviceHandler::new(
            input_device_configs
                .into_iter()
                .map_into()
                .collect::<Vec<_>>(),
        );

        Self {
            logs: vec![
                DemexLogEntry::new(DemexLogEntryType::Info(format!(
                    "demex v{} (by @matteolutz), Welcome!",
                    VERSION_STR
                ))),
                DemexLogEntry::new(DemexLogEntryType::Info(
                    "Check out https://demex.matteolutz.de to get started.".to_owned(),
                )),
            ],

            ui_locked: false,

            is_command_input_empty: true,
            command: Vec::new(),
            should_focus_command_input: false,
            command_input: String::new(),
            action_queue: ActionQueue::default(),

            encoders_tab_state: EncodersTabState::default(),
            window_handler: DemexWindowHandler::default(),
            global_fixture_select: None,
            global_sequence_select: UiEditRequest::None,

            ui_config,

            show_file,

            fixture_handler,
            preset_handler,
            updatable_handler,
            timing_handler,
            patch,

            input_device_handler,

            stats,

            texture_handles,
        }
    }

    pub fn open_new_show(&mut self) {
        rfd::FileDialog::new()
            .add_filter("demex Show-File", &["json"])
            .pick_file()
            .ok_or(DemexUiError::RuntimeError(
                "Failed show file dialog".to_string(),
            ))
            .and_then(|show_file| {
                let show: DemexShow =
                    serde_json::from_reader(std::fs::File::open(&show_file).unwrap())
                        .map_err(DemexUiError::SerdeJsonError)?;

                let input_device_configs = show.input_device_configs;

                let new_patch = {
                    let patch = self.patch.read();
                    show.patch.into_patch(patch.fixture_types().to_vec())
                };

                let (fixtures, outputs) =
                    new_patch.into_fixures_and_outputs(DemexProtoDeviceId::Controller);

                *self.fixture_handler.write() = FixtureHandler::new(fixtures, outputs, true)
                    .map_err(|err| DemexUiError::RuntimeError(err.to_string()))?;
                *self.preset_handler.write() = show.preset_handler;
                *self.updatable_handler.write() = show.updatable_handler;
                *self.timing_handler.write() = show.timing_handler;

                *self.patch.write() = new_patch;

                self.input_device_handler = DemexInputDeviceHandler::new(
                    input_device_configs
                        .into_iter()
                        .map_into()
                        .collect::<Vec<_>>(),
                );

                Ok(())
            })
            .unwrap_or_else(|e| {
                self.add_dialog_entry(DemexGlobalDialogEntry::error(&e));
            });
    }

    pub fn save_show(&mut self, ui_config: DemexShowUiConfig) {
        let save_result = {
            let preset_handler_lock = self.preset_handler.read();
            let updatable_handler_lock = self.updatable_handler.read();
            let timing_handler_lock = self.timing_handler.read();
            let patch_lock = self.patch.read();

            let show = DemexShow {
                preset_handler: preset_handler_lock.clone(),
                updatable_handler: updatable_handler_lock.clone(),
                timing_handler: timing_handler_lock.clone(),
                input_device_configs: self
                    .input_device_handler
                    .devices()
                    .iter()
                    .map(|d| d.config().clone())
                    .collect::<Vec<_>>(),
                patch: SerializablePatch::from_patch(&patch_lock),
                ui_config,
            };

            let save_file = if let Some(show_file) = self.show_file.as_ref() {
                Ok(show_file.clone())
            } else if let Some(save_file) = rfd::FileDialog::new()
                .add_filter("demex Show-File", &["json"])
                .save_file()
            {
                Ok(save_file)
            } else {
                Err(DemexUiError::RuntimeError(
                    "No save file selected".to_owned(),
                ))
            };

            save_file.and_then(|save_file| {
                serde_json::to_writer(std::fs::File::create(&save_file).unwrap(), &show)
                    .map_err(DemexUiError::SerdeJsonError)?;
                Ok(save_file)
            })
        };

        if let Err(e) = save_result {
            self.add_dialog_entry(DemexGlobalDialogEntry::error(&e));
        } else if let Ok(save_file) = save_result {
            self.add_dialog_entry(DemexGlobalDialogEntry::info(
                format!("Show saved to {}", save_file.display()).as_str(),
            ));
            self.show_file = Some(save_file);
        }
    }

    pub fn run_and_handle_action(
        &mut self,
        action: DeferredAction,
        ui_config: &DemexShowUiConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (action, issued_at) = (action.action, action.issued_at);

        match &action {
            Action::ClearAll => {
                self.global_fixture_select = None;
                self.window_handler.clear();
            }
            Action::HomeAll => {
                let mut fixture_handler_lock = self.fixture_handler.write();
                let preset_handler_lock = self.preset_handler.read();
                let mut updatable_handler_lock = self.updatable_handler.write();

                updatable_handler_lock
                    .executors_stop_all(&mut fixture_handler_lock, &preset_handler_lock);
            }
            Action::Save => {
                self.save_show(ui_config.clone());
            }
            Action::Test(cmd) => match cmd.as_str() {
                _ => self.add_dialog_entry(DemexGlobalDialogEntry::error(
                    &DemexUiError::RuntimeError(format!("Unknown test command: \"{}\"", cmd)),
                )),
            },
            _ => {}
        }

        let now = std::time::Instant::now();

        let result = action
            .run(
                &mut self.fixture_handler.write(),
                &mut self.preset_handler.write(),
                FixtureSelectorContext::new(&self.global_fixture_select),
                &mut self.updatable_handler.write(),
                &mut self.input_device_handler,
                &mut self.timing_handler.write(),
                &self.patch.read(),
                issued_at,
            )
            .inspect(|result| {
                self.logs
                    .push(DemexLogEntry::new(DemexLogEntryType::ActionEntrySuccess(
                        action.clone(),
                        result.clone(),
                    )))
            })
            .inspect_err(|err| {
                self.logs
                    .push(DemexLogEntry::new(DemexLogEntryType::ActionEntryFailed(
                        action.clone(),
                        err.to_string(),
                    )))
            })?;

        log::debug!(
            "Execution of action {:?} took {:.2?}",
            action,
            now.elapsed()
        );

        match result {
            ActionRunResult::Warn(warn) => {
                self.add_dialog_entry(DemexGlobalDialogEntry::warn(warn.as_str()));
            }
            ActionRunResult::Info(info) => {
                self.add_dialog_entry(DemexGlobalDialogEntry::info(info.as_str()));
            }
            ActionRunResult::InfoWithLink(info, link) => {
                self.add_dialog_entry(DemexGlobalDialogEntry::info_with_link(
                    info.as_str(),
                    link.as_str(),
                ));
            }
            ActionRunResult::EditWindow(edit_window) => {
                let demex_edit_window = DemexWindow::Edit(edit_window);

                self.window_handler.add_window(demex_edit_window);
            }
            ActionRunResult::UpdateSelectedFixtures(selection) => {
                self.global_fixture_select = selection;
            }
            ActionRunResult::Lock => self.ui_locked = true,
            ActionRunResult::Default => {}
        }

        Ok(())
    }
}
