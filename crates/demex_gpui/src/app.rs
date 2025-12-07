use std::path::PathBuf;

use gdtf::fixture_type::FixtureType;

use crate::{
    engine::{showfile::DemexShowFileManager, state::DemexUiState},
    ui2::{self, assets::Assets},
};

pub struct DemexAppArgs {
    pub fixture_types: Vec<FixtureType>,
    pub showfile_path: Option<PathBuf>,

    pub touchscreen_mode: bool,
    pub additional_viewports: usize,
}

pub mod actions {
    use crate::{
        engine::showfile::{self, DemexShowFileManager},
        ui2::wm::{WindowManager, app::WindowManagerAppExt},
    };
    use gpui::{App, KeyBinding, Menu, MenuItem, SystemMenuType};

    gpui::actions!(demex, [Quit, Save, SaveAs, Open, Reload]);
    pub(super) fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

        cx.bind_keys([KeyBinding::new("secondary-s", Save, None)]);
        cx.bind_keys([KeyBinding::new("secondary-shift-s", SaveAs, None)]);
        cx.bind_keys([KeyBinding::new("secondary-o", Open, None)]);

        cx.on_action::<Quit>(|_, cx| cx.quit());
        cx.on_action::<Save>(|_, cx| {
            if DemexShowFileManager::current_file_path(cx)
                .read(cx)
                .is_some()
            {
                DemexShowFileManager::save(None, cx, |path, cx| {
                    cx.update_wm(|wm, cx| {
                        wm.push_success(format!("Saved to \"{}\"", path.display()), cx)
                    });
                });
            } else {
                cx.dispatch_action(&SaveAs);
            }
        });
        cx.on_action::<SaveAs>(|_, cx| {
            let current_filename = DemexShowFileManager::current_file_path(cx)
                .read(cx)
                .as_ref()
                .and_then(|buf| buf.file_name())
                .and_then(|name| name.to_str())
                .map(|name| name.to_string());

            cx.spawn(async |cx| {
                let file = showfile::dialog::save_showfile_dialog(current_filename).await;
                let Some(file) = file else {
                    return;
                };
                let _ = cx.update(|cx| {
                    DemexShowFileManager::save(Some(file.path().into()), cx, |path, cx| {
                        cx.update_wm(|wm, cx| {
                            wm.push_success(format!("Saved to \"{}\"", path.display()), cx)
                        });
                    })
                });
            })
            .detach();
        });
        cx.on_action::<Open>(|_, cx| {
            cx.spawn(async |cx| {
                let file = showfile::dialog::open_showfile_dialog().await;
                let Some(file) = file else {
                    return;
                };
                let _ = cx.update(|cx| {
                    WindowManager::inspect_error(
                        DemexShowFileManager::load_showfile(file.path(), cx),
                        "Failed to load showfile: ",
                        cx,
                    )
                });
            })
            .detach();
        });
        cx.on_action::<Reload>(|_, cx| {
            // TODO: prompt this
            let _ = WindowManager::inspect_error(
                DemexShowFileManager::reload(cx),
                "Failed to reload showfile: ",
                cx,
            );
        });

        init_menus(cx);
    }

    pub fn init_menus(cx: &mut App) {
        cx.set_menus(vec![Menu {
            name: "demex".into(),
            items: vec![
                MenuItem::os_submenu("Services", SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
        }]);
    }
}

#[derive(Default)]
pub struct DemexApp {}

impl DemexApp {
    pub fn run(self, args: DemexAppArgs) {
        gpui::Application::new()
            .with_assets(Assets)
            .run(move |cx: &mut gpui::App| {
                use gpui_component::{Theme, ThemeRegistry};

                use crate::ui2::{
                    config::DemexUiConfig,
                    wm::{WindowManager, dock_window::DockWindowConfig},
                };

                gpui_component::init(cx);
                ui2::init(cx).unwrap();

                actions::init(cx);

                let theme_reg = ThemeRegistry::global(cx);
                if let Some(theme) = theme_reg.themes().get("Default Dark").cloned() {
                    Theme::global_mut(cx).apply_config(&theme);
                }

                let ui_config = DemexUiConfig {
                    touchcreen_mode: args.touchscreen_mode,
                };
                cx.set_global(ui_config);

                // initialize ui state ()
                DemexUiState::init(cx);

                let wm = WindowManager::new(cx).auto_quit(true);
                cx.set_global(wm);

                DemexShowFileManager::init(args.fixture_types, cx)
                    .expect("Failed to initialize show file manager");

                WindowManager::add_dock_windows(
                    (0..(args.additional_viewports + 1)).map(|_| DockWindowConfig::default()),
                    cx,
                );

                cx.activate(true);

                let load_default = args.showfile_path.is_none_or(|showfile| {
                    DemexShowFileManager::load_showfile(showfile, cx).is_err()
                });

                if load_default {
                    DemexShowFileManager::load_empty_show(cx)
                }

                DemexUiState::start_performance_thread(cx);
            });
    }
}
