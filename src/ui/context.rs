use std::{path::PathBuf, sync::Arc};

use parking_lot::RwLock;

use crate::{
    fixture::{
        channel2::feature::feature_group::FeatureGroup, handler::FixtureHandler,
        presets::PresetHandler, selection::FixtureSelection, timing::TimingHandler,
        updatables::UpdatableHandler,
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
    show::DemexShow,
    ui::error::DemexUiError,
    utils::thread::DemexThreadStatsHandler,
};

use super::{
    log::{dialog::DemexGlobalDialogEntry, DemexLogEntry, DemexLogEntryType},
    window::{edit::DemexEditWindow, DemexWindow},
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

    pub windows: Vec<DemexWindow>,
}

impl DemexUiContext {
    pub fn add_dialog_entry(&mut self, dialog_entry: DemexGlobalDialogEntry) {
        let dialog_window = self.windows.iter_mut().find(|w| w.is_dialog());

        if let Some(dialog_window) = dialog_window {
            dialog_window.add_dialog_entry(dialog_entry.clone());
        } else {
            self.windows
                .push(DemexWindow::Dialog(vec![dialog_entry.clone()]));
        }

        self.logs
            .push(DemexLogEntry::new(DemexLogEntryType::DialogEntry(
                dialog_entry,
            )));
    }

    pub fn add_edit_window(&mut self, edit_window: DemexEditWindow) {
        let window = DemexWindow::Edit(edit_window);

        if self.windows.contains(&window) {
            return;
        }

        self.windows.push(window);
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
                self.windows.retain(|el| !el.is_dialog());
            }
            Action::HomeAll => {
                let mut updatable_handler_lock = self.updatable_handler.write();
                let mut fixture_handler_lock = self.fixture_handler.write();

                updatable_handler_lock.executors_stop_all(&mut fixture_handler_lock);

                updatable_handler_lock.faders_home_all(&mut fixture_handler_lock);
            }
            Action::Save => {
                let fixture_handler_lock = self.fixture_handler.read();
                let mut preset_handler_lock = self.preset_handler.write();
                let updatable_handler_lock = self.updatable_handler.read();
                let timing_handler_lock = self.timing_handler.read();

                *preset_handler_lock.feature_groups_mut() = FeatureGroup::default_feature_groups();

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

        println!(
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

                if !self.windows.contains(&demex_edit_window) {
                    self.windows.push(demex_edit_window);
                }
            }
            ActionRunResult::UpdateSelectedFixtures(selection) => {
                self.global_fixture_select = Some(selection);
            }
            ActionRunResult::Default => {}
        }

        Ok(())
    }
}
