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

//! Fixture definitions and builders used by GDCS.

use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::{cmp, fmt, str};

use demex_dmx::address::DmxAddress;
use gdtf::dmx_mode::LogicalChannelMaster;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::channel3::attribute::FixtureChannel3Attribute;
use crate::channel3::clamped_value::ClampedValue;
use crate::fixture::error::FixtureError;
use crate::patch::Patch;

/// A configured fixture instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub(crate) path: FixturePath,
    pub(crate) root_base_address: DmxAddress,
    pub(crate) name: String,

    pub(crate) gdtf_fixture_type_id: Uuid,
    pub(crate) gdtf_dmx_mode: String,
    pub(crate) channel_functions: HashMap<FixtureChannel3Attribute, FixtureChannelFunction>,

    pub(crate) sub_fixture_paths: Vec<FixturePath>,
}

impl Fixture {
    /// Returns the path identifying this fixture within the fixture tree.
    pub fn path(&self) -> FixturePath {
        self.path
    }

    /// Returns the root DMX base address assigned to this fixture.
    ///
    /// This is the first address occupied by the fixture in the DMX
    /// universe (addresses occupied by sub-fixtures are derived from this).
    pub fn base_address(&self) -> DmxAddress {
        self.root_base_address
    }

    /// Returns the name for the fixture instance.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the paths of any sub-fixtures contained by this fixture.
    pub fn sub_fixtures(&self) -> &[FixturePath] {
        &self.sub_fixture_paths
    }

    /// Returns the GDTF fixture type this instance is based on.
    pub fn gdtf_fixture_type_id(&self) -> Uuid {
        self.gdtf_fixture_type_id
    }

    /// Returns the GDTF DMX mode used by this fixture instance.
    pub fn gdtf_dmx_mode(&self) -> &str {
        &self.gdtf_dmx_mode
    }

    /// Get the channel function associated with the given attribute.
    ///
    /// Returns `None` if the attribute is not present on this fixture.
    pub fn channel_function(
        &self,
        attribute: &FixtureChannel3Attribute,
    ) -> Option<&FixtureChannelFunction> {
        self.channel_functions.get(attribute)
    }

    /// Get all channel functions for this fixture.
    pub fn channel_functions(
        &self,
    ) -> impl Iterator<Item = (&FixtureChannel3Attribute, &FixtureChannelFunction)> {
        self.channel_functions.iter()
    }

    pub fn has_attribute(&self, attribute: &FixtureChannel3Attribute) -> bool {
        self.channel_functions().any(|(attr, _)| attr == attribute)
    }

    pub fn get_attributes_recursive(&self, patch: &Patch) -> HashSet<FixtureChannel3Attribute> {
        let mut attrs: HashSet<FixtureChannel3Attribute> =
            self.channel_functions.keys().copied().collect();

        for child_path in self.sub_fixtures() {
            let Ok(child) = patch.fixture(child_path) else {
                continue;
            };

            attrs.extend(child.get_attributes_recursive(patch));
        }

        attrs
    }

    pub fn replace_parent(&mut self, old_parent: &FixturePath, new_parent: FixturePath) {
        self.path = self.path.replace_parent(old_parent, new_parent);
        self.sub_fixture_paths.iter_mut().for_each(|p| {
            *p = p.replace_parent(old_parent, new_parent);
        });
        self.channel_functions.iter_mut().for_each(|(_, cf)| {
            match &mut cf.kind {
                FixtureChannelFunctionKind::Virtual { relations } => {
                    relations
                        .iter_mut()
                        .filter(|rel| rel.fixture_path.starts_with(old_parent))
                        .for_each(|rel| {
                            rel.fixture_path =
                                rel.fixture_path.replace_parent(old_parent, new_parent)
                        });
                }
                _ => {}
            };
        });
    }
}

/// Describes how a fixture attribute maps to DMX channel values.
///
/// A channel function defines whether the attribute is controlled by
/// physical DMX addresses or derived virtually from other attributes,
/// and the range of values it accepts (min/max) and its default value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FixtureChannelFunction {
    pub(crate) kind: FixtureChannelFunctionKind,
    pub(crate) min: ClampedValue,
    pub(crate) max: ClampedValue,
    pub(crate) default: ClampedValue,
    pub(crate) highlight: Option<ClampedValue>,

    pub(crate) sets: HashMap<String, ClampedValue>,

    pub(crate) activation_group: Option<String>,
    pub(crate) master: LogicalChannelMaster,
    pub(crate) is_initial: bool,
    pub(crate) snap: bool,
}

impl FixtureChannelFunction {
    /// Returns the kind of this channel function (physical or virtual).
    pub fn kind(&self) -> &FixtureChannelFunctionKind {
        &self.kind
    }

    /// The minimum value (inclusive) supported by this channel function.
    pub fn min(&self) -> ClampedValue {
        self.min
    }

    /// The maximum value (inclusive) supported by this channel function.
    pub fn max(&self) -> ClampedValue {
        self.max
    }

    /// The default value (projected in the channel function) for this attribute when no explicit value is set.
    pub fn default(&self) -> ClampedValue {
        self.default
    }

    /// The highlight value (projected in the channel function) for this attribute
    pub fn highlight(&self) -> Option<ClampedValue> {
        self.highlight
    }

    /// Whether this channel function should snap instead of fade
    pub fn snap(&self) -> bool {
        self.snap
    }

    pub fn unprojected_default(&self) -> ClampedValue {
        self.unproject(self.default)
    }

    pub fn unprojected_highlight(&self) -> Option<ClampedValue> {
        self.highlight.map(|highlight| self.unproject(highlight))
    }

    pub(crate) fn project_cf_value(
        value: ClampedValue,
        min: ClampedValue,
        max: ClampedValue,
    ) -> ClampedValue {
        let range = max.as_f32() - min.as_f32();
        (min.as_f32() + range * value.as_f32()).into()
    }

    pub(crate) fn unproject_cf_value(
        value: ClampedValue,
        min: ClampedValue,
        max: ClampedValue,
    ) -> ClampedValue {
        let range = max.as_f32() - min.as_f32();
        ((value.as_f32() - min.as_f32()) / range).into()
    }

    /// Project the value (0.0..=1.0) to the range of this channel function.
    pub fn project(&self, value: ClampedValue) -> ClampedValue {
        Self::project_cf_value(value, self.min, self.max)
    }

    /// Unproject the value from the range of this channel function to (0.0..=1.0).
    pub fn unproject(&self, value: ClampedValue) -> ClampedValue {
        Self::unproject_cf_value(value, self.min, self.max)
    }

    /// Get a channel set by name for this channel function.
    pub fn set(&self, name: &str) -> Option<ClampedValue> {
        self.sets.get(name).copied()
    }
}

/// Specifies whether an attribute is mapped to physical DMX channels or is
/// computed virtually from other attributes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FixtureChannelFunctionKind {
    /// A physical channel mapping addresses to a channel functions.
    /// (multiple are used for fine-controlled channel functions like Pan or Tilt).
    Physical {
        /// DMX addresses.
        addresses: Vec<DmxAddress>,
    },

    /// A virtual mapping derived from relationships to other fixture attributes.
    Virtual {
        /// Relations to other fixture attributes used to compute the value.
        relations: Vec<Relation>,
    },
}

/// A relation describes how a virtual attribute is derived from another
/// attribute.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Relation {
    pub(crate) kind: RelationKind,
    pub(crate) fixture_path: FixturePath,
    pub(crate) attribute: FixtureChannel3Attribute,
}

impl Relation {
    /// Creates a new `Relation`.
    pub fn new(
        kind: RelationKind,
        fixture_path: FixturePath,
        attribute: FixtureChannel3Attribute,
    ) -> Self {
        Self {
            kind,
            fixture_path,
            attribute,
        }
    }

    /// Returns the relation kind (e.g. multiply or override).
    pub fn kind(&self) -> &RelationKind {
        &self.kind
    }

    /// Returns the path to the fixture this relation references.
    pub fn fixture_path(&self) -> FixturePath {
        self.fixture_path
    }

    /// Returns the attribute on the referenced fixture used by this relation.
    pub fn attribute(&self) -> FixtureChannel3Attribute {
        self.attribute
    }
}

/// The operation used when combining a source attribute into a virtual attribute.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum RelationKind {
    /// Multiply the source attribute value with the target.
    Multiply,
    /// Override the target with the source attribute value.
    Override,
}

/// A non-zero identifier for a fixture.
///
/// `FixtureId` guarantees the inner identifier is never zero. Use
/// `FixtureId::new` to construct a validated id.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct FixtureId(NonZeroU32);

impl FixtureId {
    /// Create a new `FixtureId` from a raw `u32`.
    ///
    /// Returns `Err(Error::InvalidFixtureId)` if `id` is zero.
    pub fn new(id: u32) -> Result<Self, FixtureError> {
        NonZeroU32::new(id)
            .map(|id| Self(id))
            .ok_or(FixtureError::FixtureIdIsZero)
    }

    /// Return the underlying identifier as a `u32`.
    pub fn as_u32(&self) -> u32 {
        self.0.into()
    }
}

impl fmt::Display for FixtureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

impl str::FromStr for FixtureId {
    type Err = FixtureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s
            .parse::<u32>()
            .map_err(FixtureError::FixtureIdParseError)?;
        FixtureId::new(id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixturePathMatchLevel {
    /// The path matches exactly.
    Exact,

    /// The top-level fixture id matches.
    Root,

    /// All fixtures that are not siblings of the given path.
    NotSibling,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// A path of [FixtureId] values.
///
/// The first element is considered the "root" fixture and additional
/// elements are sub-fixtures. The maximum number of elements is [FixturePath::MAX_LEN].
pub struct FixturePath {
    ids: [FixtureId; Self::MAX_LEN],
    len: u8,
}

impl FixturePath {
    /// Maximum number of [FixtureId]s that can be stored in a [FixturePath].
    pub const MAX_LEN: usize = 8;

    /// Create a new [FixturePath] containing only the given root fixture.
    pub fn new(root_id: FixtureId) -> Self {
        let mut ids = [FixtureId::new(1).unwrap(); Self::MAX_LEN];
        ids[0] = root_id;
        FixturePath { ids, len: 1 }
    }

    /// Append a fixture identifier to the end of the path.
    ///
    /// # Panics
    ///
    /// Panics if the path already contains [FixturePath::MAX_LEN] elements.
    pub fn push(&mut self, id: FixtureId) {
        let len = self.len();
        assert!(
            len < Self::MAX_LEN,
            "FixturePath capacity exceeded (max {})",
            Self::MAX_LEN
        );
        self.ids[len] = id;
        self.len = (len + 1) as u8;
    }

    /// Returns the number of fixtures in this path.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if this path contains only the root fixture.
    pub fn is_root_fixture(&self) -> bool {
        self.len == 1
    }

    /// Returns the number of sub-fixtures (excluding the root).
    pub fn sub_len(&self) -> usize {
        assert!(self.len() >= 1, "FixturePath must have at least a root");
        self.len() - 1
    }

    /// Returns the root [FixtureId] of the path.
    pub fn root(&self) -> FixtureId {
        self.ids[0]
    }

    /// Returns the last [FixtureId] in the path.
    pub fn last(&self) -> FixtureId {
        let l = self.len();
        assert!(l >= 1, "FixturePath must have at least a root");
        self.ids[l - 1]
    }

    /// Borrow the path as a slice of [FixtureId]s.
    pub fn as_slice(&self) -> &[FixtureId] {
        &self.ids[..self.len()]
    }

    /// Returns an iterator over the fixture identifiers in the path.
    pub fn iter(&self) -> std::slice::Iter<'_, FixtureId> {
        self.as_slice().iter()
    }

    /// Replace the last element of the path with `sub_id`.
    pub fn replace_last(&mut self, sub_id: FixtureId) {
        let l = self.len();
        assert!(l >= 1, "FixturePath must have at least a root");
        self.ids[l - 1] = sub_id;
    }

    /// Return a new [FixturePath] with `part` appended.
    pub fn extended_with(mut self, part: FixtureId) -> Self {
        self.push(part);
        self
    }

    pub fn extend(&mut self, other: &[FixtureId]) {
        assert!(
            (self.len as usize + other.len()) <= Self::MAX_LEN,
            "FixturePath is too long"
        );

        for (ix, id) in other.into_iter().enumerate() {
            self.ids[self.len as usize + ix] = *id;
        }
        self.len += other.len() as u8;
    }

    pub fn with_parent(&self, parent: Self) -> Self {
        let last = self.last();
        parent.extended_with(last)
    }

    pub fn replace_parent(self, old_parent: &Self, mut new_parent: Self) -> Self {
        assert!(
            self.starts_with(old_parent),
            "FixturePath does not start with the old parent"
        );

        let rest = &self.ids[old_parent.len()..self.len()];

        new_parent.extend(rest);
        new_parent
    }

    /// Returns `true` if `self` contains `path` as a prefix.
    pub fn starts_with(&self, path: &Self) -> bool {
        let path_len = path.len();
        if path_len > self.len() {
            return false;
        }
        &self.as_slice()[..path_len] == path.as_slice()
    }

    pub fn matches(&self, path: &Self, level: FixturePathMatchLevel) -> bool {
        match level {
            FixturePathMatchLevel::Exact => self == path,
            FixturePathMatchLevel::Root => self.root() == path.root(),
            FixturePathMatchLevel::NotSibling => {
                let path_len = path.len();
                if path_len == self.len() {
                    self == path
                } else if path_len < self.len() {
                    &self.as_slice()[..path_len] == path.as_slice()
                } else {
                    &path.as_slice()[..self.len()] == self.as_slice()
                }
            }
        }
    }
}

impl AsRef<[FixtureId]> for FixturePath {
    fn as_ref(&self) -> &[FixtureId] {
        self.as_slice()
    }
}

impl From<FixtureId> for FixturePath {
    fn from(id: FixtureId) -> Self {
        Self::new(id)
    }
}

impl From<&[FixtureId]> for FixturePath {
    fn from(slice: &[FixtureId]) -> Self {
        assert!(
            slice.len() <= Self::MAX_LEN,
            "FixturePath slice length {} exceeds capacity {}",
            slice.len(),
            Self::MAX_LEN
        );
        let mut ids = [FixtureId::new(1).unwrap(); Self::MAX_LEN];
        for (i, v) in slice.iter().enumerate() {
            ids[i] = *v;
        }
        FixturePath {
            ids,
            len: slice.len() as u8,
        }
    }
}

impl IntoIterator for FixturePath {
    type Item = FixtureId;
    type IntoIter = std::vec::IntoIter<FixtureId>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().to_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a FixturePath {
    type Item = &'a FixtureId;
    type IntoIter = std::slice::Iter<'a, FixtureId>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl cmp::PartialOrd for FixturePath {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for FixturePath {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let a = self.as_slice();
        let b = other.as_slice();
        for (x, y) in a.iter().zip(b.iter()) {
            let ord = x.cmp(y);
            if ord != cmp::Ordering::Equal {
                return ord;
            }
        }
        a.len().cmp(&b.len())
    }
}
impl fmt::Display for FixturePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for id in self.as_slice() {
            if !first {
                write!(f, ".")?;
            }
            write!(f, "{}", id)?;
            first = false;
        }
        Ok(())
    }
}

impl fmt::Debug for FixturePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FixturePath(")?;
        fmt::Display::fmt(self, f)?;
        write!(f, ")")
    }
}

impl str::FromStr for FixturePath {
    type Err = FixtureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.is_empty() {
            return Err(FixtureError::FixturePathIsEmpty);
        }

        if parts.len() > Self::MAX_LEN {
            return Err(FixtureError::FixturePathHasTooManyParts);
        }
        let mut ids = [FixtureId::new(1).unwrap(); Self::MAX_LEN];
        for (i, part) in parts.iter().enumerate() {
            ids[i] = FixtureId::from_str(part)?;
        }
        Ok(Self {
            ids,
            len: parts.len() as u8,
        })
    }
}

impl serde::Serialize for FixturePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use std::fmt::Write;
        let mut s = String::new();
        write!(&mut s, "{}", self).unwrap();
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for FixturePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FixturePathVisitor;

        impl<'de> serde::de::Visitor<'de> for FixturePathVisitor {
            type Value = FixturePath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing a FixturePath")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::str::FromStr;
                FixturePath::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(FixturePathVisitor)
    }
}

#[macro_export]
macro_rules! fpath {
    ( $first:literal $(, $rest:literal )* $(,)? ) => {{
        let mut p = $crate::fixture::FixturePath::new(
            $crate::fixture::FixtureId::new($first).unwrap()
        );
        $( p.push($crate::state::fixture::FixtureId::new($rest).unwrap()); )*
        p
    }};
    ( $first:expr $(, $rest:expr )* $(,)? ) => {{
        #[allow(unused_mut)]
        let mut p = $crate::fixture::FixturePath::new($first);
        $( p.push($rest); )*
        p
    }};
}
