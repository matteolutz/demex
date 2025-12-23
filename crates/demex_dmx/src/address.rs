use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DmxAddress {
    pub universe: u16,
    pub channel: u16,
}

impl DmxAddress {
    pub fn new(universe: u16, channel: u16) -> Self {
        Self { universe, channel }
    }
}

impl DmxAddress {
    pub fn with_channel_offset(self, offset: i32) -> Option<Self> {
        let current_abs = (self.universe as i64 - 1) * 512 + (self.channel as i64 - 1);
        let total = current_abs + offset as i64;

        let universe_idx = total.div_euclid(512); // may be negative
        let new_channel_zero = total.rem_euclid(512) as u16; // 0..=511

        let target_universe_id = 1 + universe_idx;
        if target_universe_id < 1 || target_universe_id > u16::MAX as i64 {
            return None;
        }

        let new_channel = new_channel_zero + 1; // 1..=512
        let target_universe_id = target_universe_id as u16;

        Some(Self {
            universe: target_universe_id,
            channel: new_channel,
        })
    }
}

impl std::fmt::Display for DmxAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.universe, self.channel)
    }
}
