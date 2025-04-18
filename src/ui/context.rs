use std::{path::PathBuf, sync::Arc};

use parking_lot::RwLock;

use crate::{
    fixture::{
        handler::FixtureHandler, presets::PresetHandler, selection::FixtureSelection,
        timing::TimingHandler, updatables::UpdatableHandler,
    },
    input::DemexInputDeviceHandler,
    lexer::token::Token,
    parser::{
        nodes::{
            action::{result::ActionRunResult, Action},
            fixture_selector::FixtureSelectorContext,
        },
        Parser2,
    },
    show::{ui::DemexShowUiConfig, DemexShow},
    ui::error::DemexUiError,
    utils::thread::DemexThreadStatsHandler,
};

use super::{
    dlog::{dialog::DemexGlobalDialogEntry, DemexLogEntry, DemexLogEntryType},
    window::{DemexWindow, DemexWindowHandler},
};

pub type SaveShowFn =
    fn(DemexShow, Option<&PathBuf>) -> Result<PathBuf, Box<dyn std::error::Error>>;

pub struct DemexUiContext {
    pub command_input: String,
    pub is_command_input_empty: bool,

    pub fixture_handler: Arc<RwLock<FixtureHandler>>,
    pub preset_handler: Arc<RwLock<PresetHandler>>,
    pub updatable_handler: Arc<RwLock<UpdatableHandler>>,
    pub timing_handler: Arc<RwLock<TimingHandler>>,

    pub global_fixture_select: Option<FixtureSelection>,
    pub command: Vec<Token>,

    pub stats: Arc<RwLock<DemexThreadStatsHandler>>,

    pub logs: Vec<DemexLogEntry>,

    pub macro_execution_queue: Vec<Action>,

    pub show_file: Option<PathBuf>,
    pub save_show: SaveShowFn,

    pub gm_slider_val: u8,

    pub input_device_handler: DemexInputDeviceHandler,

    // pub windows: Vec<DemexWindow>,
    pub window_handler: DemexWindowHandler,

    pub ui_config: DemexShowUiConfig,
}

impl DemexUiContext {
    pub fn add_dialog_entry(&mut self, dialog_entry: DemexGlobalDialogEntry) {
        self.window_handler.add_dialog_window(dialog_entry.clone());

        self.logs
            .push(DemexLogEntry::new(DemexLogEntryType::DialogEntry(
                dialog_entry,
            )));
    }

    pub fn run_cmd(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

        self.run_and_handle_action(&action)
    }

    pub fn run_and_handle_action(
        &mut self,
        action: &Action,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                updatable_handler_lock
                    .faders_home_all(&mut fixture_handler_lock, &preset_handler_lock);
            }
            Action::Save => {
                let fixture_handler_lock = self.fixture_handler.read();
                let mut preset_handler_lock = self.preset_handler.write();
                let updatable_handler_lock = self.updatable_handler.read();
                let timing_handler_lock = self.timing_handler.read();

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
                    patch: fixture_handler_lock.patch().clone(),
                    ui_config: self.ui_config.clone(),
                };

                let save_result = (self.save_show)(show, self.show_file.as_ref());

                drop(fixture_handler_lock);
                drop(updatable_handler_lock);
                drop(preset_handler_lock);
                drop(timing_handler_lock);

                if let Err(e) = save_result {
                    self.add_dialog_entry(DemexGlobalDialogEntry::error(e.as_ref()));
                } else if let Ok(save_file) = save_result {
                    self.add_dialog_entry(DemexGlobalDialogEntry::info(
                        format!("Show saved to {}", save_file.display()).as_str(),
                    ));
                    self.show_file = Some(save_file);
                }
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
                self.global_fixture_select = Some(selection);
            }
            ActionRunResult::Default => {}
        }

        Ok(())
    }
}
