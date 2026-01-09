use demex_core::{
    command::parser::nodes::{
        action::{
            Action,
            functions::{go_function::ExecutorGoArgs, set_function::SetFixturePresetArgs},
        },
        fixture_selector::FixtureSelector,
    },
    engine::comm::ExecutorSequenceRequest,
    pool::PoolType,
    presets::preset::FixturePresetId,
};
use gpui::App;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::panels::pool::pool_button::PoolButton,
};

pub fn handle_pool_item_click(pool_type: PoolType, pool_item_id: u32, cx: &mut App) {
    let engine = DemexEngineHandler::engine(cx);

    match pool_type {
        PoolType::Preset(preset_type) => engine.exec_ui(Action::SetFixturePreset(
            SetFixturePresetArgs::current(FixturePresetId::new(preset_type, pool_item_id)),
        )),
        PoolType::Executor => engine.exec_ui(Action::ExecutorGo(ExecutorGoArgs {
            executor_id: pool_item_id,
        })),
        PoolType::Sequence => {
            DemexUiState::selected_sequence(cx).update(cx, |selected_sequence, cx| {
                *selected_sequence = Some(pool_item_id);
                cx.notify();
            })
        }
        PoolType::Group => engine.exec_ui(Action::FixtureSelector(FixtureSelector::group(
            pool_item_id,
        ))),
        PoolType::Macro => engine.exec_ui(Action::RunMacro(pool_item_id)),
        _ => {}
    }
}

pub fn apply_pool_type_to_button(
    pool_type: PoolType,
    button: PoolButton,
    pool_item_id: u32,
) -> PoolButton {
    match pool_type {
        PoolType::Executor => button.action("Edit Seq", move |_, cx| {
            DemexEngineHandler::send(
                cx,
                ExecutorSequenceRequest {
                    executor_id: pool_item_id,
                },
                |seq, cx| {
                    if let Some(seq) = seq {
                        DemexUiState::selected_sequence(cx).update(cx, |selected_sequence, cx| {
                            *selected_sequence = Some(seq.id);
                            cx.notify();
                        });
                    }
                },
            );
        }),
        _ => button,
    }
}
