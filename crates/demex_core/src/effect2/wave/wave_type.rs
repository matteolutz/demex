use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WaveType {
    #[default]
    Triangle,
    Square,

    Bezier,
}
