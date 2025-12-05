use std::{fs, path::PathBuf};

use demex_core::{
    engine::{comm::ShowRequest, error::DemexEngineError},
    show::DemexShow,
};
use gdtf::fixture_type::FixtureType;
use gpui::{App, AppContext, BorrowAppContext, Entity, Global};
use gpui_component::notification::Notification;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::wm::app::WindowManagerAppExt,
};

pub struct DemexShowFileManager {
    global_fixture_types: Vec<FixtureType>,

    current_file_path: Entity<Option<PathBuf>>,
}

impl Global for DemexShowFileManager {}

impl DemexShowFileManager {
    pub fn current_file_path(cx: &App) -> Entity<Option<PathBuf>> {
        let manager: &Self = cx.global();
        manager.current_file_path.clone()
    }
}

impl DemexShowFileManager {
    pub fn init(
        global_fixture_types: Vec<FixtureType>,
        showfile_path: Option<PathBuf>,
        cx: &mut App,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut s = Self {
            global_fixture_types,
            current_file_path: cx.new(|_| showfile_path.clone()),
        };

        DemexUiState::init(cx);

        let show = showfile_path
            .as_ref()
            .and_then(|path| Self::parse_show_file(&path).ok());
        if show.is_none() {
            log::warn!("Couldn't load initial showfile, falling back to default.");
        }

        s.load_show(show.unwrap_or_default(), cx)?;

        cx.set_global(s);

        DemexUiState::start_performance_thread(cx);

        Ok(())
    }

    pub fn load_show(&mut self, show: DemexShow, cx: &mut App) -> Result<(), DemexEngineError> {
        let frontend_state = DemexEngineHandler::init(self.global_fixture_types.clone(), show, cx)?;

        cx.update_global(|ui_state: &mut DemexUiState, cx| {
            ui_state.load_frontend_state(frontend_state, cx)
        });

        Ok(())
    }

    fn parse_show_file(path: &PathBuf) -> Result<DemexShow, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        serde_json::from_reader(file).map_err(|err| err.into())
    }

    fn write_show_to_file(
        show: &DemexShow,
        path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = fs::File::create(path)?;
        serde_json::to_writer(file, &show).map_err(|err| err.into())
    }

    pub fn load_empty_show(&mut self, cx: &mut App) -> Result<(), DemexEngineError> {
        let show = DemexShow::default();
        let res = self.load_show(show, cx)?;

        self.current_file_path
            .update(cx, |current_path, _| *current_path = None);

        Ok(res)
    }

    pub fn load_showfile(
        &mut self,
        path: impl Into<PathBuf>,
        cx: &mut App,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.into();

        let show = Self::parse_show_file(&path)?;
        let res = self.load_show(show, cx).map_err(|err| Box::new(err))?;

        self.current_file_path
            .update(cx, |current_path, _| *current_path = Some(path));

        Ok(res)
    }

    pub fn save_show(
        &mut self,
        show: DemexShow,
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

    #[allow(unused)]
    pub fn open(path: impl Into<PathBuf>, cx: &mut App) -> Result<(), Box<dyn std::error::Error>> {
        return Err("Not yet implemented".into());
        cx.update_global(|this: &mut Self, cx| this.load_showfile(path, cx))
    }

    pub fn save(save_as: Option<PathBuf>, cx: &mut App) {
        DemexEngineHandler::send(cx, ShowRequest {}, |show: DemexShow, cx: &mut App| {
            let res = cx.update_global(|this: &mut Self, cx| this.save_show(show, save_as, cx));

            cx.update_wm(|wm, cx| {
                wm.push_notifcation(
                    match res {
                        Ok(path) => {
                            Notification::success(format!("Saved to \"{}\"", path.display()))
                        }
                        Err(err) => Notification::error(format!("Failed to save show: {}", err)),
                    },
                    cx,
                );
            });
        });
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
