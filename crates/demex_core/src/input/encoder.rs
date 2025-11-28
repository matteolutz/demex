use crate::{
    EncoderChannels, channel3::channel_value::FixtureChannelValue3Update,
    command::parser::nodes::fixture_selector::FixtureSelectorContext, patch::Patch,
    presets::PresetHandler, state::fixture_state_handler::FixtureStateHandler,
    timing::TimingHandler,
};

pub(crate) fn get_global_encoder_value(
    encoder_idx: u32,
    fixture_selector_context: FixtureSelectorContext,
    fixture_handler: &FixtureStateHandler,
    preset_handler: &PresetHandler,
    timing_handler: &TimingHandler,
    encoder_channels: Option<&EncoderChannels>,
    patch: &Patch,
) -> Option<f32> {
    return Some(0.0);
    /*
    let fixture_selection = fixture_selector_context.current_fixture()?;

    let master_fixture = fixture_selection.master_fixture(patch)?;

    let (_, channel_map) = encoder_channels?.get(encoder_idx as usize)?;
    let channel = channel_map
        .get(&master_fixture.type_and_mode_hash())?
        .first()?;

    let value = master_fixture
        .get_value(patch.fixture_types(), channel)
        .ok()?;

    let (_, value) = value.get_as_display(
        patch,
        master_fixture,
        channel,
        preset_handler,
        timing_handler,
    );

    Some(value)
    */
}

pub(crate) fn handle_global_encoder_change(
    encoder_idx: u32,
    value: f32,
    fixture_selector_context: FixtureSelectorContext,
    fixture_handler: &mut FixtureStateHandler,
    encoder_channels: Option<&EncoderChannels>,
    patch: &Patch,
) {
    return;

    /*
    let Some(fixture_selection) = fixture_selector_context.current_fixture() else {
        return;
    };

    let fixtures = fixture_handler.selected_fixtures_mut(fixture_selection);

    let Some((_, channels)) =
        encoder_channels.and_then(|channels| channels.get(encoder_idx as usize))
    else {
        return;
    };

    for fixture in fixtures {
        let Some(channels) = channels.get(&fixture.type_and_mode_hash()) else {
            continue;
        };

        for channel in channels {
            let _ = fixture.update_programmer_value(
                patch.fixture_types(),
                channel,
                FixtureChannelValue3Update::Value(value),
            );
        }
    }
    */
}
