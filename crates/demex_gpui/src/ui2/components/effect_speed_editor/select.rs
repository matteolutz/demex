use demex_core::effect::speed::{EffectSpeed, EffectSpeedScale, EffectSpeedSyncMode};
use gpui_component::{IndexPath, select::SelectItem};
use itertools::Itertools;
use strum::IntoEnumIterator;

#[derive(Debug, Copy, Clone)]
pub struct EffectSpeedSelectItem(EffectSpeed);

impl EffectSpeedSelectItem {
    pub fn index_of(effect_speed: &EffectSpeed) -> IndexPath {
        IndexPath::new(
            EffectSpeed::iter()
                .find_position(|es| {
                    std::mem::discriminant(es) == std::mem::discriminant(effect_speed)
                })
                .unwrap()
                .0,
        )
    }
}

impl From<EffectSpeed> for EffectSpeedSelectItem {
    fn from(speed: EffectSpeed) -> Self {
        EffectSpeedSelectItem(speed)
    }
}

impl SelectItem for EffectSpeedSelectItem {
    type Value = EffectSpeed;

    fn title(&self) -> gpui::SharedString {
        match &self.0 {
            EffectSpeed::Bpm(_) => "BPM".into(),
            EffectSpeed::SpeedMaster { .. } => "Speedmaster".into(),
        }
    }

    fn value(&self) -> &Self::Value {
        &self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EffectSpeedSyncModeSelectItem(EffectSpeedSyncMode);

impl EffectSpeedSyncModeSelectItem {
    pub fn index_of(sync_mode: &EffectSpeedSyncMode) -> IndexPath {
        IndexPath::new(
            EffectSpeedSyncMode::iter()
                .find_position(|sm| sm == sync_mode)
                .unwrap()
                .0,
        )
    }
}

impl From<EffectSpeedSyncMode> for EffectSpeedSyncModeSelectItem {
    fn from(value: EffectSpeedSyncMode) -> Self {
        Self(value)
    }
}

impl SelectItem for EffectSpeedSyncModeSelectItem {
    type Value = EffectSpeedSyncMode;

    fn title(&self) -> gpui::SharedString {
        format!("{:?}", self.0).into()
    }

    fn value(&self) -> &Self::Value {
        &self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EffectSpeedScaleSelectItem(EffectSpeedScale);

impl EffectSpeedScaleSelectItem {
    pub fn index_of(speed_scale: &EffectSpeedScale) -> IndexPath {
        IndexPath::new(
            EffectSpeedScale::iter()
                .find_position(|speed_s| speed_s == speed_scale)
                .unwrap()
                .0,
        )
    }
}

impl From<EffectSpeedScale> for EffectSpeedScaleSelectItem {
    fn from(value: EffectSpeedScale) -> Self {
        Self(value)
    }
}

impl SelectItem for EffectSpeedScaleSelectItem {
    type Value = EffectSpeedScale;

    fn title(&self) -> gpui::SharedString {
        match self.0 {
            EffectSpeedScale::Div128 => "/128",
            EffectSpeedScale::Div64 => "/64",
            EffectSpeedScale::Div32 => "/32",
            EffectSpeedScale::Div16 => "/16",
            EffectSpeedScale::Div8 => "/8",
            EffectSpeedScale::Div4 => "/4",
            EffectSpeedScale::Div2 => "/2",

            EffectSpeedScale::One => "One",

            EffectSpeedScale::Mul2 => "*2",
            EffectSpeedScale::Mul4 => "*4",
            EffectSpeedScale::Mul8 => "*8",
            EffectSpeedScale::Mul16 => "*16",
            EffectSpeedScale::Mul32 => "*32",
            EffectSpeedScale::Mul64 => "*64",
            EffectSpeedScale::Mul128 => "*128",
        }
        .into()
    }

    fn value(&self) -> &Self::Value {
        &self.0
    }
}
