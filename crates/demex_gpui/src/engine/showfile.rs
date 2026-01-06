use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use demex_core::{engine::comm::ShowRequest, show::DemexShow, utils::version::VERSION_STR};
use gdtf::fixture_type::FixtureType;
use gpui::{App, AppContext, AsyncApp, BorrowAppContext, Entity, Global, Task, Timer};
use serde::{Deserialize, Serialize};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::wm::{app::WindowManagerAppExt, dock_window::DockWindowConfig},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DemexUiShowConfig {
    dock_windows: Vec<DockWindowConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DemexUiShow {
    engine: DemexShow,

    #[serde(default)]
    ui: DemexUiShowConfig,

    version: String,
}

impl From<(DemexShow, DemexUiShowConfig)> for DemexUiShow {
    fn from((engine, ui): (DemexShow, DemexUiShowConfig)) -> Self {
        Self {
            engine,
            ui,
            version: VERSION_STR.to_string(),
        }
    }
}

pub struct DemexShowFileManager {
    global_fixture_types: Vec<FixtureType>,

    current_file_path: Entity<Option<PathBuf>>,

    last_autosave: Entity<Option<Instant>>,
    _autosave: Task<()>,
}

impl Global for DemexShowFileManager {}

impl DemexShowFileManager {
    pub fn current_file_path(cx: &App) -> Entity<Option<PathBuf>> {
        let manager: &Self = cx.global();
        manager.current_file_path.clone()
    }

    pub fn last_autosave(cx: &App) -> Entity<Option<Instant>> {
        let manager: &Self = cx.global();
        manager.last_autosave.clone()
    }
}

impl DemexShowFileManager {
    async fn autosave(cx: &mut AsyncApp) {
        let autosave_duration = Duration::from_secs(20);

        loop {
            Timer::after(autosave_duration).await;

            let has_file =
                cx.read_global(|this: &Self, cx| this.current_file_path.read(cx).is_some());

            if has_file.ok().is_none_or(|has_file| !has_file) {
                continue;
            }

            let _ = cx.update(|cx| {
                Self::save(None, cx, |_, cx| {
                    cx.update_global(|this: &mut Self, cx| {
                        this.last_autosave.update(cx, |last_auto_save, cx| {
                            *last_auto_save = Some(Instant::now());
                            cx.notify();
                        })
                    })
                })
            });
        }
    }
}

impl DemexShowFileManager {
    pub fn init(
        global_fixture_types: Vec<FixtureType>,
        cx: &mut App,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let s = Self {
            global_fixture_types,
            current_file_path: cx.new(|_| None),

            _autosave: cx.spawn(Self::autosave),
            last_autosave: cx.new(|_| None),
        };

        let frontend_state =
            DemexEngineHandler::init(DemexShow::default(), s.global_fixture_types.clone(), cx)?;
        cx.update_global(|ui_state: &mut DemexUiState, cx| {
            ui_state.load_frontend_state(frontend_state, cx)
        });

        cx.set_global(s);

        Ok(())
    }

    fn load_show(ui_show: DemexUiShow, cx: &mut App) {
        cx.defer(|cx| {
            cx.update_global(|this: &mut Self, cx| {
                let frontend_state = cx.update_global(|handler: &mut DemexEngineHandler, _| {
                    handler.update_show(ui_show.engine, this.global_fixture_types.clone())
                });

                cx.update_global(|ui_state: &mut DemexUiState, cx| {
                    ui_state.load_frontend_state(frontend_state, cx)
                });

                cx.update_wm(|wm, cx| wm.update_dock_window_configs(ui_show.ui.dock_windows, cx));
            });
        });
    }

    fn parse_show_file(path: &PathBuf) -> Result<DemexUiShow, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        serde_json::from_reader(file)
            .inspect_err(|err| log::error!("Error parsing showfile: {}", err))
            .map_err(|err| err.into())
    }

    fn write_show_to_file(
        show: &DemexUiShow,
        path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = fs::File::create(path)?;
        serde_json::to_writer(file, &show).map_err(|err| err.into())
    }

    pub fn load_empty_show(cx: &mut App) {
        Self::load_show(DemexUiShow::default(), cx);

        cx.defer(|cx| {
            cx.update_global(|this: &mut Self, cx| {
                this.current_file_path.update(cx, |current_path, cx| {
                    *current_path = None;
                    cx.notify();
                });
            });
        });
    }

    pub fn load_showfile(
        path: impl Into<PathBuf>,
        cx: &mut App,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.into();

        let ui_show = Self::parse_show_file(&path)?;
        Self::load_show(ui_show, cx);

        cx.defer(|cx| {
            cx.update_global(|this: &mut Self, cx| {
                this.current_file_path.update(cx, |current_path, cx| {
                    *current_path = Some(path);
                    cx.notify();
                });
            });
        });

        Ok(())
    }

    pub fn save_show(
        &mut self,
        show: DemexUiShow,
        save_as: Option<PathBuf>,
        cx: &mut App,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        match save_as {
            Some(path) => {
                Self::write_show_to_file(&show, &path)?;
                self.current_file_path
                    .update(cx, |current_path, _| *current_path = Some(path.clone()));
                Ok(path)
            }
            None => {
                let Some(current_path) = self.current_file_path.read(cx) else {
                    return Err("No file path available".into());
                };

                Self::write_show_to_file(&show, &current_path)?;
                Ok(current_path.clone())
            }
        }
    }

    pub fn save<F>(save_as: Option<PathBuf>, cx: &mut App, on_save: F)
    where
        F: FnOnce(&PathBuf, &mut App) + Send + 'static,
    {
        let on_save = Box::new(on_save);

        DemexEngineHandler::send(cx, ShowRequest {}, move |show: DemexShow, cx: &mut App| {
            let dock_windows = cx.wm().dock_window_configs(cx).collect::<Vec<_>>();
            let ui_config = DemexUiShowConfig { dock_windows };

            let res = cx.update_global(|this: &mut Self, cx| {
                this.save_show((show, ui_config).into(), save_as, cx)
            });

            match res {
                Ok(path) => on_save(&path, cx),
                Err(err) => cx.update_wm(|wm, cx| {
                    wm.push_error(format!("Failed to save show: {}", err), cx);
                }),
            };
        });
    }

    pub fn reload(cx: &mut App) -> Result<(), Box<dyn std::error::Error>> {
        let path = cx.global::<Self>().current_file_path.read(cx).clone();

        let Some(path) = path else {
            return Ok(());
        };

        Self::load_showfile(path, cx)
    }
}

pub mod dialog {

    use rfd::{AsyncFileDialog, FileHandle};

    pub async fn save_showfile_dialog(current_filename: Option<String>) -> Option<FileHandle> {
        let mut builder = AsyncFileDialog::new()
            .add_filter("demex Showfile", &["json"])
            .set_directory("/");

        if let Some(filename) = current_filename {
            builder = builder.set_file_name(filename);
        }

        builder.save_file().await
    }

    pub async fn open_showfile_dialog() -> Option<FileHandle> {
        AsyncFileDialog::new()
            .add_filter("demex Showfile", &["json"])
            .set_directory("/")
            .pick_file()
            .await
    }
}
