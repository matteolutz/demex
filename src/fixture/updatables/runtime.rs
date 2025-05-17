use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum RuntimePhase {
    // Phase in degrees
    Single(f32),

    // Phase in degrees
    Range { start: f32, end: f32 },
}

impl Default for RuntimePhase {
    fn default() -> Self {
        Self::Single(0.0)
    }
}

impl RuntimePhase {
    pub fn phase(&self, offset: f32) -> f32 {
        // Offset is a float between 0.0 and 1.0
        // if we have a rnage we interpolate between start and end
        match self {
            Self::Single(phase) => *phase,
            Self::Range { start, end } => start + (end - start) * offset,
        }
    }
}
