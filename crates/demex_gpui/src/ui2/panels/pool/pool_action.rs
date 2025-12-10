use demex_core::{
    command::parser::nodes::{
        action::{
            Action,
            functions::{go_function::ExecutorGoArgs, set_function::SetFixturePresetArgs},
        },
        fixture_selector::FixtureSelector,
    },
    pool::PoolType,
    presets::preset::FixturePresetId,
};
use gpui::App;

use crate::engine::DemexEngineHandler;

pub fn handle_pool_item_click(pool_type: PoolType, pool_item_id: u32, cx: &mut App) {
    let engine = DemexEngineHandler::engine(cx);

    match pool_type {
        PoolType::Preset(preset_type) => engine.exec_ui(Action::SetFixturePreset(
            SetFixturePresetArgs::current(FixturePresetId::new(preset_type, pool_item_id)),
        )),
        PoolType::Executor => engine.exec_ui(Action::ExecutorGo(ExecutorGoArgs {
            executor_id: pool_item_id,
        })),
        PoolType::Group => engine.exec_ui(Action::FixtureSelector(FixtureSelector::group(
            pool_item_id,
        ))),
        PoolType::Macro => engine.exec_ui(Action::RunMacro(pool_item_id)),
        _ => {}
    }
}
