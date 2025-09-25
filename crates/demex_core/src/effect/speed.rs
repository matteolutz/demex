use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum EffectSpeedScale {
    Div128,
    Div64,
    Div32,
    Div16,
    Div8,
    Div4,
    Div2,

    #[default]
    One,

    Mul2,
    Mul4,
    Mul8,
    Mul16,
    Mul32,
    Mul64,
    Mul128,
}

impl EffectSpeedScale {
    pub fn scale_value(&self) -> f32 {
        match self {
            EffectSpeedScale::Div128 => 1.0 / 128.0,
            EffectSpeedScale::Div64 => 1.0 / 64.0,
            EffectSpeedScale::Div32 => 1.0 / 32.0,
            EffectSpeedScale::Div16 => 1.0 / 16.0,
            EffectSpeedScale::Div8 => 1.0 / 8.0,
            EffectSpeedScale::Div4 => 1.0 / 4.0,
            EffectSpeedScale::Div2 => 1.0 / 2.0,
            EffectSpeedScale::One => 1.0,
            EffectSpeedScale::Mul2 => 2.0,
            EffectSpeedScale::Mul4 => 4.0,
            EffectSpeedScale::Mul8 => 8.0,
            EffectSpeedScale::Mul16 => 16.0,
            EffectSpeedScale::Mul32 => 32.0,
            EffectSpeedScale::Mul64 => 64.0,
            EffectSpeedScale::Mul128 => 128.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum EffectSpeedSyncMode {
    #[default]
    None,

    SyncBeatFrac,
    SyncBeat,
}

impl EffectSpeedSyncMode {
    pub fn is_synced(&self) -> bool {
        !matches!(self, EffectSpeedSyncMode::None)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum EffectSpeed {
    Bpm(f32),
    SpeedMaster {
        id: u32,
        scale: EffectSpeedScale,
        sync: EffectSpeedSyncMode,
    },
}

impl Default for EffectSpeed {
    fn default() -> Self {
        EffectSpeed::Bpm(120.0)
    }
}
