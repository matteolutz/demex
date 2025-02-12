use std::{sync::Arc, time};

use parking_lot::RwLock;

use crate::{
    fixture::{
        channel::{
            value::FixtureChannelValue, FIXTURE_CHANNEL_COLOR_ID,
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
        },
        effect::FixtureChannelEffect,
        handler::FixtureHandler,
        presets::PresetHandler,
        updatables::UpdatableHandler,
    },
    lexer::token::Token,
    parser::nodes::{
        action::{result::ActionRunResult, Action},
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
    show::DemexShow,
    ui::error::DemexUiError,
    utils::thread::DemexThreadStatsHandler,
};

use super::{
    log::{dialog::DemexGlobalDialogEntry, DemexLogEntry, DemexLogEntryType},
    tabs::layout_view_tab::LayoutViewContext,
    window::DemexWindow,
};

pub struct DemexUiContext {
    pub fixture_handler: Arc<RwLock<FixtureHandler>>,
    pub preset_handler: Arc<RwLock<PresetHandler>>,
    pub updatable_handler: Arc<RwLock<UpdatableHandler>>,

    pub global_fixture_select: Option<FixtureSelector>,
    pub command: Vec<Token>,

    pub stats: Arc<RwLock<DemexThreadStatsHandler>>,

    pub logs: Vec<DemexLogEntry>,

    pub macro_execution_queue: Vec<Action>,

    pub save_show: fn(DemexShow) -> Result<(), Box<dyn std::error::Error>>,

    pub layout_view_context: LayoutViewContext,
    pub gm_slider_val: u8,

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
                let preset_handler_lock = self.preset_handler.read();
                let updatable_handler_lock = self.updatable_handler.read();

                let show = DemexShow {
                    preset_handler: preset_handler_lock.clone(),
                    updatable_handler: updatable_handler_lock.clone(),
                    patch: fixture_handler_lock.patch().clone(),
                };

                let save_result = (self.save_show)(show);

                drop(fixture_handler_lock);
                drop(updatable_handler_lock);
                drop(preset_handler_lock);

                if let Err(e) = save_result {
                    self.add_dialog_entry(DemexGlobalDialogEntry::error(e.as_ref()));
                } else {
                    self.add_dialog_entry(DemexGlobalDialogEntry::info("Show saved"));
                }
            }
            Action::Test(cmd) => match cmd.as_str() {
                "effect" => {
                    let _ = self
                        .fixture_handler
                        .write()
                        .fixture(1)
                        .unwrap()
                        .set_channel_value(
                            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
                            FixtureChannelValue::Effect {
                                effect: FixtureChannelEffect::PairFigureEight {
                                    speed: 1.0,
                                    center_a: 0.5,
                                    center_b: 0.5,
                                },
                                started: Some(time::Instant::now()),
                            },
                        );

                    let _ = self
                        .fixture_handler
                        .write()
                        .fixture(1)
                        .unwrap()
                        .set_channel_value(
                            FIXTURE_CHANNEL_COLOR_ID,
                            FixtureChannelValue::Effect {
                                effect: FixtureChannelEffect::QuadrupleHueRotate { speed: 1.0 },
                                started: Some(time::Instant::now()),
                            },
                        );
                }
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
            ActionRunResult::UpdateSelectedFixtures(fixture_selector) => {
                self.global_fixture_select = Some(fixture_selector)
            }
            ActionRunResult::Default => {}
        }

        Ok(())
    }
}
