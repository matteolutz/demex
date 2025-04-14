use serde::{Deserialize, Serialize};

use super::channel_type::FixtureChannelType;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum FixtureChannelModifier {
    Invert(FixtureChannelType),
    Max {
        channel_type: FixtureChannelType,
        max: u8,
    },
    Min {
        channel_type: FixtureChannelType,
        min: u8,
    },
}

pub trait FixtureChannelModifierTrait {
    fn apply(&self, channel_type: FixtureChannelType, value: u8) -> u8;
}

fn channel_type_matches_or_fine(
    modifier_channel_type: FixtureChannelType,
    channel_type: FixtureChannelType,
) -> bool {
    modifier_channel_type == channel_type
        || channel_type
            .get_fine()
            .is_some_and(|fine| fine == modifier_channel_type)
}

impl FixtureChannelModifierTrait for Vec<FixtureChannelModifier> {
    fn apply(&self, channel_type: FixtureChannelType, mut value: u8) -> u8 {
        for modifier in self {
            match modifier {
                FixtureChannelModifier::Invert(modifier_channel_type)
                    if channel_type_matches_or_fine(*modifier_channel_type, channel_type) =>
                {
                    value = 255 - value;
                }
                FixtureChannelModifier::Max {
                    channel_type: modifier_channel_type,
                    max,
                } if channel_type_matches_or_fine(*modifier_channel_type, channel_type) => {
                    value = value.min(*max);
                }
                FixtureChannelModifier::Min {
                    channel_type: modifier_channel_type,
                    min,
                } if channel_type_matches_or_fine(*modifier_channel_type, channel_type) => {
                    value = value.max(*min);
                }
                _ => {}
            }
        }

        value
    }
}
