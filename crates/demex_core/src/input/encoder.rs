use crate::{
    EncoderChannels, command::parser::nodes::fixture_selector::FixtureSelectorContext,
    patch::Patch, presets::PresetHandler, state::fixture_state_handler::FixtureStateHandler,
    timing::TimingHandler,
};

pub(crate) fn get_global_encoder_value(
    _encoder_idx: u32,
    _fixture_selector_context: FixtureSelectorContext,
    _fixture_handler: &FixtureStateHandler,
    _preset_handler: &PresetHandler,
    _timing_handler: &TimingHandler,
    _encoder_channels: Option<&EncoderChannels>,
    _patch: &Patch,
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
    _encoder_idx: u32,
    _value: f32,
    _fixture_selector_context: FixtureSelectorContext,
    _fixture_handler: &mut FixtureStateHandler,
    _encoder_channels: Option<&EncoderChannels>,
    _patch: &Patch,
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
