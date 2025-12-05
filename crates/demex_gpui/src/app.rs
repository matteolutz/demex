use std::path::PathBuf;

use gdtf::fixture_type::FixtureType;

use crate::{
    engine::showfile::DemexShowFileManager,
    ui2::{self, assets::Assets},
};

pub struct DemexAppArgs {
    pub fixture_types: Vec<FixtureType>,
    pub showfile_path: Option<PathBuf>,
    pub touchscreen_mode: bool,
}

pub mod actions {
    use crate::engine::showfile::{self, DemexShowFileManager};
    use gpui::{App, KeyBinding, Menu, MenuItem, SystemMenuType};

    gpui::actions!(demex, [Quit, Save, SaveAs, Open]);
    pub(super) fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

        cx.bind_keys([KeyBinding::new("secondary-s", Save, None)]);
        cx.bind_keys([KeyBinding::new("secondary-shift-s", SaveAs, None)]);
        cx.bind_keys([KeyBinding::new("secondary-o", Open, None)]);

        cx.on_action::<Quit>(|_, cx| cx.quit());
        cx.on_action::<Save>(|_, cx| DemexShowFileManager::save(None, cx));
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
                let _ = cx.update(|cx| DemexShowFileManager::save(Some(file.path().into()), cx));
            })
            .detach();
        });
        cx.on_action::<Open>(|_, cx| {
            cx.spawn(async |cx| {
                let file = showfile::dialog::open_showfile_dialog().await;
                let Some(file) = file else {
                    return;
                };
                let _ = cx.update(|cx| DemexShowFileManager::open(file.path(), cx));
            })
            .detach();
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

                cx.activate(true);

                let ui_config = DemexUiConfig {
                    touchcreen_mode: args.touchscreen_mode,
                };
                cx.set_global(ui_config);

                DemexShowFileManager::init(args.fixture_types, args.showfile_path, cx)
                    .expect("Failed to initialize show file manager");

                let wm = WindowManager::new(cx).auto_quit(true);
                cx.set_global(wm);

                WindowManager::add_dock_window(DockWindowConfig::default(), cx);
            });
    }
}
