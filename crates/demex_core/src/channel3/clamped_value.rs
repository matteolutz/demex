/*
 * This file is part of zeevonk (https://github.com/BaukeWestendorp/zeevonk).
 *
 * Original work:
 *   Copyright (C) 2025 Bauke Westendorp
 *
 * Modifications:
 *   Copyright (C) 2025 Matteo Lutz
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 */

use std::{fmt, num, str};

/// A clamped value.
///
/// ClampedValue represents a floating-point value constrained to the range
/// [0.0, 1.0]. All operations automatically clamp values to this valid range.
#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct ClampedValue(f32);

impl ClampedValue {
    /// The minimum allowed value (0.0).
    pub const MIN: f32 = 0.0;

    /// The maximum allowed value (1.0).
    pub const MAX: f32 = 1.0;

    /// Creates a new ClampedValue with the specified value.
    ///
    /// The value is automatically clamped to the range [0.0, 1.0].
    #[inline]
    pub const fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    /// Sets the value of this ClampedValue.
    ///
    /// The value is automatically clamped to the range [0.0, 1.0].
    #[inline]
    pub fn set(&mut self, value: f32) {
        self.0 = value.clamp(Self::MIN, Self::MAX);
    }

    /// Returns the underlying f32 value.
    ///
    /// The returned value is guaranteed to be in the range [0.0, 1.0].
    #[inline]
    pub fn as_f32(self) -> f32 {
        self.0
    }

    /// Performs linear interpolation between this value and another.
    #[inline]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(Self::MIN, Self::MAX);
        Self::new(self.0 * (1.0 - t) + other.0 * t)
    }

    /// Converts the value to a 1-byte representation (u8).
    #[inline]
    pub fn to_u8(&self) -> u8 {
        (self.0 * 255.0).round().clamp(0.0, 255.0) as u8
    }

    /// Converts the value to a 2-byte representation ([u8; 2]), big-endian.
    #[inline]
    pub fn to_u16_bytes(&self) -> [u8; 2] {
        let val = (self.0 * 65535.0).round().clamp(0.0, 65535.0) as u16;
        val.to_be_bytes()
    }

    /// Converts the value to a 3-byte representation ([u8; 3]), big-endian.
    #[inline]
    pub fn to_u24_bytes(&self) -> [u8; 3] {
        let val = (self.0 * 16777215.0).round().clamp(0.0, 16777215.0) as u32;
        [
            ((val >> 16) & 0xFF) as u8,
            ((val >> 8) & 0xFF) as u8,
            (val & 0xFF) as u8,
        ]
    }

    /// Converts the value to a 4-byte representation ([u8; 4]), big-endian.
    #[inline]
    pub fn to_u32_bytes(&self) -> [u8; 4] {
        let val = (self.0 * 4294967295.0).round().clamp(0.0, 4294967295.0) as u32;
        val.to_be_bytes()
    }
}

impl fmt::Display for ClampedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f32> for ClampedValue {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<ClampedValue> for f32 {
    fn from(value: ClampedValue) -> Self {
        value.0
    }
}

impl From<ClampedValue> for f64 {
    fn from(value: ClampedValue) -> Self {
        value.0 as f64
    }
}

impl From<ClampedValue> for u8 {
    fn from(value: ClampedValue) -> Self {
        (value.0 * (u8::MAX as f32)) as u8
    }
}

impl str::FromStr for ClampedValue {
    type Err = num::ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}
