use crate::{
    fixture::{
        channel3::channel_value::FixtureChannelValue3Discrete, handler::FixtureHandler,
        patch::Patch, presets::PresetHandler, timing::TimingHandler,
    },
    parser::nodes::fixture_selector::FixtureSelectorContext,
    ui::context::EncoderChannels,
};

pub(crate) fn get_global_encoder_value(
    encoder_idx: u32,
    fixture_selector_context: FixtureSelectorContext,
    fixture_handler: &FixtureHandler,
    preset_handler: &PresetHandler,
    timing_handler: &TimingHandler,
    encoder_channels: Option<&EncoderChannels>,
    patch: &Patch,
) -> Option<f32> {
    let fixture_selection = fixture_selector_context.current_fixture()?;

    let master_fixture = fixture_selection.master_fixture(fixture_handler)?;

    let (_, channel_map) = encoder_channels?.get(encoder_idx as usize)?;
    let channel = channel_map
        .get(&master_fixture.type_and_mode_hash())?
        .first()?;

    let value = master_fixture
        .get_value(patch.fixture_types(), channel)
        .ok()?;

    let (_, value) = value.get_as_discrete(
        master_fixture,
        patch.fixture_types(),
        channel,
        preset_handler,
        timing_handler,
    );

    Some(value)
}

pub(crate) fn handle_global_encoder_change(
    encoder_idx: u32,
    value: f32,
    fixture_selector_context: FixtureSelectorContext,
    fixture_handler: &mut FixtureHandler,
    encoder_channels: Option<&EncoderChannels>,
    patch: &Patch,
) {
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
                FixtureChannelValue3Discrete::Value(value),
            );
        }
    }
}
