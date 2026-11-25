use serde::{Deserialize, Serialize};

use crate::{
    channel3::attribute::FixtureChannel3Attribute,
    command::parser::nodes::{
        action::{
            Action, ActionIssuer,
            functions::set_function::{SetAttributeValue, SetAttributeValueArgs},
            queue::ActionQueue,
        },
        fixture_selector::FixtureSelector,
    },
    event::DemexEvent,
    input::{
        DemexInputDeviceUpdateArgs, control::DemexInputDeviceControlDelegate,
        error::DemexInputDeviceError, event::DemexInputDeviceEncoderUpdate, message::EncoderValue,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DemexInputEncoder {
    GlobalEncoder { encoder_idx: u32 },
}

impl Default for DemexInputEncoder {
    fn default() -> Self {
        Self::GlobalEncoder { encoder_idx: 0 }
    }
}

impl DemexInputEncoder {
    pub fn handle_change(
        &self,
        value: EncoderValue,
        action_queue: &mut ActionQueue,
        encoder_attributes: &[FixtureChannel3Attribute],
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::GlobalEncoder { encoder_idx } => {
                if let Some(&attribute) = encoder_attributes.get(*encoder_idx as usize) {
                    action_queue.enqueue_now(
                        Action::SetAttributeValue(SetAttributeValueArgs {
                            fixture_selector: FixtureSelector::current_fixtures_selected(),
                            attribute,
                            attribute_value: Some(match value {
                                EncoderValue::Absolute(value) => {
                                    SetAttributeValue::Absolute(value.into())
                                }
                                EncoderValue::RelativeChange(change) => {
                                    SetAttributeValue::RelativChange(change.into())
                                }
                            }),
                        }),
                        ActionIssuer::InputDevice,
                    );
                }
            }
        }

        Ok(())
    }

    pub fn value(&self, _args: DemexInputDeviceUpdateArgs) -> Result<f32, DemexInputDeviceError> {
        match self {
            Self::GlobalEncoder { encoder_idx: _ } => Ok(0.0), // TODO
        }
    }
}

impl DemexInputDeviceControlDelegate for DemexInputEncoder {
    type Update = DemexInputDeviceEncoderUpdate;

    fn map_event(
        &self,
        _args: crate::input::DemexInputDeviceUpdateArgs,
        _event: &DemexEvent,
    ) -> Result<Option<Self::Update>, crate::input::error::DemexInputDeviceError> {
        // TODO

        Ok(None)
    }
}
