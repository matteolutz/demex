pub enum DemexInputDeviceMessage {
    ButtonPressed(u32),
    ButtonReleased(u32),

    FaderValueChanged(u32, f32),
    FaderValuesChanged(Vec<(u32, f32)>),
}
