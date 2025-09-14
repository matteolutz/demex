use crate::{
    fixture::{
        channel3::channel_value::FixtureChannelValue3Discrete, handler::FixtureHandler,
        patch::Patch,
    },
    parser::nodes::fixture_selector::FixtureSelectorContext,
    ui::context::EncoderChannels,
};

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
                &channel,
                FixtureChannelValue3Discrete::Value(value),
            );
        }
    }
}
