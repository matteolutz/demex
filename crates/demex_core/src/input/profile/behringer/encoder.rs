#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BehringerXTouchCompactEncoderMode {
    Single,
    Pan,
    Fan,
    Spread,
    Trim,
}

impl BehringerXTouchCompactEncoderMode {
    pub fn value(&self) -> u8 {
        match self {
            BehringerXTouchCompactEncoderMode::Single => 0,
            BehringerXTouchCompactEncoderMode::Pan => 1,
            BehringerXTouchCompactEncoderMode::Fan => 2,
            BehringerXTouchCompactEncoderMode::Spread => 3,
            BehringerXTouchCompactEncoderMode::Trim => 4,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BehringerXTouchCompactButtonLedMode {
    Off,
    Blink,
    On,
}

impl BehringerXTouchCompactButtonLedMode {
    pub fn velocity(&self) -> u8 {
        match self {
            BehringerXTouchCompactButtonLedMode::Off => 0,
            BehringerXTouchCompactButtonLedMode::Blink => 1,
            BehringerXTouchCompactButtonLedMode::On => 127,
        }
    }
}
