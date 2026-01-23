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
 * it under the terms of the MIT License as published by
 * the Free Software Foundation, version 3.
 */

use std::{collections::HashMap, str::FromStr};

use demex_dmx::address::DmxAddress;
use gdtf::{dmx_mode::RelationType, geometry::AnyGeometry};

use crate::{
    channel3::{attribute::FixtureChannel3Attribute, clamped_value::ClampedValue},
    fixture::{
        Fixture, FixtureChannelFunction, FixtureChannelFunctionKind, FixtureId, FixturePath,
        GdtfFixturePatch, Relation, RelationKind, error::FixtureError,
    },
    patch::FixtureTypeList,
};

/// Helper for building the fixture tree from a GDTF fixture type + DMX mode.
///
/// The builder walks the nested geometry tree, constructs fixtures and their channel
/// functions (physical or virtual), and resolves relations for virtual channels after the
/// first pass.
pub struct FixtureBuilder<'a> {
    root_id: FixtureId,
    name: String,
    address: DmxAddress,

    should_collapse: bool,

    gdtf_fixture_type: &'a gdtf::fixture_type::FixtureType,
    gdtf_dmx_mode: &'a gdtf::dmx_mode::DmxMode,

    fixtures: Vec<Fixture>,

    // Keeps track of how many siblings have been created at each depth of the geometry tree.
    // The top of the stack corresponds to the current parent whose children are being enumerated.
    sibling_count_stack: Vec<u32>,

    // Map a channel function (identified by geometry + indices + fixture path) to the
    // fixture path where it lives for quick lookup when resolving relations.
    channel_function_map: HashMap<ChannelFunctionId, FixturePath>,

    // Virtual channel functions are registered on the first pass, but their relations
    // depend on being able to find followers across the whole fixture set, so we store
    // them to resolve after the initial construction.
    unresolved_virtual_channels: Vec<(ChannelFunctionId, FixtureChannel3Attribute)>,
    // defaults: HashSet<(DmxAddress, u8)>,
}

impl<'a> FixtureBuilder<'a> {
    pub fn from_patch(
        fixture: GdtfFixturePatch,
        fixture_types: &'a FixtureTypeList,
    ) -> Result<Self, FixtureError> {
        let fixture_type = fixture_types
            .iter()
            .find(|ft| ft.fixture_type_id == fixture.fixture_type_id)
            .ok_or(FixtureError::GdtfFixtureTypeNotFound(
                fixture.fixture_type_id,
            ))?;

        let dmx_mode = fixture_type.dmx_mode(fixture.dmx_mode()).ok_or_else(|| {
            FixtureError::GdtfFixtureDmxModeNotFound(fixture.fixture_type_dmx_mode)
        })?;

        Ok(Self::new(
            FixtureId::new(fixture.id).unwrap(),
            fixture.name,
            DmxAddress::new(fixture.universe, fixture.start_address),
            fixture_type,
            dmx_mode,
        ))
    }

    pub fn new(
        root_id: FixtureId,
        name: String,
        address: DmxAddress,
        gdtf_fixture_type: &'a gdtf::fixture_type::FixtureType,
        gdtf_dmx_mode: &'a gdtf::dmx_mode::DmxMode,
    ) -> Self {
        Self {
            root_id,
            name,
            address,

            should_collapse: false,

            gdtf_fixture_type,
            gdtf_dmx_mode,

            fixtures: Vec::new(),
            sibling_count_stack: Vec::new(),
            channel_function_map: HashMap::new(),
            unresolved_virtual_channels: Vec::new(),
        }
    }

    pub fn should_collapse(mut self, should_collapse: bool) -> Self {
        self.should_collapse = should_collapse;
        self
    }

    pub fn build_fixture_tree(mut self) -> Result<Vec<Fixture>, FixtureError> {
        // Find the root geometry for the chosen DMX mode and start the recursive building.
        let root_geometry = self.get_root_geometry()?.clone();
        let root_path = FixturePath::new(self.root_id);
        self.fixtures = self.fixtures_from_geometry(root_path, &root_geometry);

        // After building all fixtures and registering virtual channels, resolve their relations.
        self.resolve_virtual_channels();

        // collapse tree
        if self.should_collapse {
            self.collapse_fixtures();
        }

        Ok(self.fixtures)
    }

    fn collapse_fixtures(&mut self) {
        let fixtures = &mut self.fixtures;

        let mut idx = 0;
        loop {
            if idx >= fixtures.len() {
                break;
            }

            // let fixture = &mut fixtures[idx];
            let fixture_path = fixtures[idx].path();

            // Fixture has no or more than one direct sub fixture
            if fixtures[idx].sub_fixtures().is_empty() || fixtures[idx].sub_fixtures().len() > 1 {
                // collapsed_fixtures.push(fixture);
                idx += 1;
                continue;
            }

            let sub_fixture = fixtures[idx + 1].clone();

            // We can collapse the channel functions if the child fixture has no channel functions that are already defined in this fixture.
            if fixtures[idx]
                .channel_functions
                .keys()
                .filter(|parent_key| sub_fixture.channel_functions.contains_key(parent_key))
                .count()
                > 0
            {
                idx += 1;
                continue;
            }

            // Extend this fixture's channel functions with the child fixture's channel functions
            fixtures[idx]
                .channel_functions
                .extend(
                    sub_fixture
                        .channel_functions
                        .into_iter()
                        .map(|(attr, mut cf)| {
                            match &mut cf.kind {
                                FixtureChannelFunctionKind::Virtual { relations } => {
                                    relations
                                        .iter_mut()
                                        .filter(|rel| {
                                            rel.fixture_path.starts_with(&sub_fixture.path)
                                        })
                                        .for_each(|rel| {
                                            rel.fixture_path = rel
                                                .fixture_path
                                                .replace_parent(&sub_fixture.path, fixture_path)
                                        });
                                }
                                _ => {}
                            };
                            (attr, cf)
                        }),
                );

            // Our only child fixture is located at idx + 1 (BFS)
            fixtures.remove(idx + 1);
            fixtures[idx + 1..]
                .iter_mut()
                .filter(|f| f.path.starts_with(&sub_fixture.path))
                .for_each(|f| {
                    f.replace_parent(&sub_fixture.path, fixture_path);
                });

            fixtures[idx].sub_fixture_paths =
                Self::collect_direct_sub_paths(&fixture_path, &fixtures);
        }
    }

    fn get_root_geometry(&self) -> Result<&gdtf::geometry::Geometry, FixtureError> {
        self.gdtf_dmx_mode
            .geometry(&self.gdtf_fixture_type)
            .ok_or_else(|| FixtureError::GdtfFixtureDmxModeGeometryNotFound(self.root_id.as_u32()))
    }

    fn fixtures_from_geometry(
        &mut self,
        sub_fixture_path: FixturePath,
        geometry: &gdtf::geometry::Geometry,
    ) -> Vec<Fixture> {
        self.sibling_count_stack.push(0);

        let fixtures = match geometry {
            gdtf::geometry::Geometry::Reference(reference) => {
                self.fixture_from_reference_geometry(sub_fixture_path, reference)
            }
            geom => self.fixture_from_geometry(sub_fixture_path, geom),
        };

        self.sibling_count_stack.pop();

        fixtures
    }

    fn fixture_from_geometry(
        &mut self,
        sub_fixture_path: FixturePath,
        geometry: &gdtf::geometry::Geometry,
    ) -> Vec<Fixture> {
        // Root fixture uses the provided fixture name, children use the geometry name.
        let name = if sub_fixture_path.is_root_fixture() {
            self.name.clone()
        } else {
            geometry
                .name()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "<no name>".to_string())
        };

        let geometry_name = geometry
            .name()
            .unwrap_or_else(|| todo!("figure out what a `None` value for a name should do"));

        self.create_sub_fixture(sub_fixture_path, name, geometry_name, geometry_name, 0)
    }

    fn fixture_from_reference_geometry(
        &mut self,
        sub_fixture_path: FixturePath,
        reference_geometry: &gdtf::geometry::ReferenceGeometry,
    ) -> Vec<Fixture> {
        // Reference geometries may introduce DMX address offsets via breaks.
        if reference_geometry.breaks.len() > 1 {
            log::warn!("multiple breaks not yet supported!");
        }

        let geometry_address_offset = match reference_geometry.breaks.get(0) {
            Some(b) => b.dmx_offset.absolute() as i32 - 1,
            None => 0,
        };

        let geometry_name = reference_geometry.name().unwrap();
        let referenced_geometry_name = reference_geometry.geometry.as_ref().unwrap();

        self.create_sub_fixture(
            sub_fixture_path,
            geometry_name.to_string(),
            &geometry_name,
            &referenced_geometry_name,
            geometry_address_offset,
        )
    }

    fn create_sub_fixture(
        &mut self,
        path: FixturePath,
        name: String,
        geometry: &gdtf::values::Name,
        referenced_geometry: &gdtf::values::Name,
        geometry_address_offset: i32,
    ) -> Vec<Fixture> {
        // Look up the nested geometry definition in the fixture type.
        let Some(referenced_geometry) =
            self.gdtf_fixture_type.nested_geometry(&referenced_geometry)
        else {
            todo!("fixure out what to do with a `None` geometry");
        };

        // Build child fixtures first (they will push/pop their own sibling counters).
        let sub_fixtures = self.collect_child_fixtures(&path, referenced_geometry);

        // Collect only the immediate children paths for this fixture's metadata.
        let sub_fixture_paths = Self::collect_direct_sub_paths(&path, &sub_fixtures);

        // Build channel functions for this referenced geometry (physical or virtual).
        let channel_functions = self.create_channel_functions(
            path,
            geometry,
            referenced_geometry.name().unwrap(),
            geometry_address_offset,
        );

        /*
        // If there's only one child fixture, we can try to collapse it's channel functions into this fixture.
        if self.should_collapse && sub_fixture_paths.len() == 1 {
            let sub_fixture = sub_fixtures
                .iter()
                .find(|f| f.path.len() == path.len() + 1)
                .unwrap();

            // We can collapse the channel functions if the child fixture has no channel functions that are already defined in this fixture.
            if channel_functions
                .keys()
                .filter(|parent_key| sub_fixture.channel_functions.contains_key(parent_key))
                .count()
                == 0
            {
                // Extend this fixture's channel functions with the child fixture's channel functions
                channel_functions.extend(sub_fixture.channel_functions.clone());

                let sub_fixture_path = sub_fixture.path();
                sub_fixtures.retain_mut(|f| {
                    let is_immediate_sub_fixture = f.path.len() == path.len() + 1;
                    f.replace_parent(&sub_fixture_path, path);
                    !is_immediate_sub_fixture
                });

                let cf_map = self
                    .channel_function_map
                    .extract_if(|key, _| key.fixture_path.starts_with(&sub_fixture_path))
                    .collect::<Vec<_>>();
                self.channel_function_map
                    .extend(cf_map.into_iter().map(|(mut key, value)| {
                        key.fixture_path = key.fixture_path.replace_parent(&sub_fixture_path, path);
                        (key, value)
                    }));

                self.unresolved_virtual_channels
                    .iter_mut()
                    .filter(|(cf_id, _)| cf_id.fixture_path.starts_with(&sub_fixture_path))
                    .for_each(|(cf_id, _)| {
                        cf_id.fixture_path =
                            cf_id.fixture_path.replace_parent(&sub_fixture_path, path);
                    });

                sub_fixture_paths = Self::collect_direct_sub_paths(&path, &sub_fixtures);
            }
        }
        */

        let gdtf_dmx_mode_name = self
            .gdtf_dmx_mode
            .name
            .as_ref()
            .expect("dmx mode should exist for name as we just found it")
            .to_string();

        let mut fixtures = vec![Fixture {
            path,
            root_base_address: self.address,
            name,
            gdtf_fixture_type_id: self.gdtf_fixture_type.fixture_type_id,
            gdtf_dmx_mode: gdtf_dmx_mode_name,
            channel_functions,
            sub_fixture_paths,
        }];

        fixtures.extend(sub_fixtures);
        fixtures
    }

    fn collect_child_fixtures(
        &mut self,
        path: &FixturePath,
        geometry: &gdtf::geometry::Geometry,
    ) -> Vec<Fixture> {
        let mut sub_fixtures = Vec::new();

        for child_geometry in geometry.children() {
            // Peek the current sibling count for this depth; it will be incremented only when
            // we actually add a fixture for this child.
            let sibling_count = {
                let last = self.sibling_count_stack.last_mut().unwrap();
                *last
            };

            let sub_fixture_path = path.extended_with(FixtureId::new(sibling_count + 1).unwrap());
            let fixtures_for_child = self.fixtures_from_geometry(sub_fixture_path, child_geometry);

            if fixtures_for_child.is_empty() {
                continue;
            }

            // Only include this sub-fixture (and its descendants) if the top-level
            // fixture for this geometry has children or channel functions.
            let parent_fixture = &fixtures_for_child[0];
            if parent_fixture.channel_functions.is_empty()
                && parent_fixture.sub_fixture_paths.is_empty()
            {
                continue;
            }

            // Only increment sibling count if we actually add a fixture
            let last = self.sibling_count_stack.last_mut().unwrap();
            *last += 1;

            sub_fixtures.extend(fixtures_for_child);
        }

        sub_fixtures
    }

    fn collect_direct_sub_paths(path: &FixturePath, sub_fixtures: &[Fixture]) -> Vec<FixturePath> {
        sub_fixtures
            .iter()
            .map(|f| f.path())
            .filter(|sub_path| sub_path.len() == path.len() + 1)
            .collect()
    }

    fn attribute_from_cf(
        &self,
        cf: &gdtf::dmx_mode::ChannelFunction,
    ) -> Option<(Option<String>, FixtureChannel3Attribute)> {
        let Some(gdtf_attribute) = cf.attribute(&self.gdtf_fixture_type) else {
            return None;
        };

        let attribute = gdtf_attribute
            .name
            .as_ref()
            // Unwrapping here is safe, as from_str for Attribute cannot fail.
            .map(|attribute| FixtureChannel3Attribute::from_str(&*attribute).unwrap());

        attribute.map(|attribute| {
            (
                gdtf_attribute
                    .activation_group(&self.gdtf_fixture_type.attribute_definitions)
                    .and_then(|ag| ag.name.as_ref())
                    .map(|name| name.to_string()),
                attribute,
            )
        })
    }

    fn create_channel_functions(
        &mut self,
        path: FixturePath,
        geometry: &gdtf::values::Name,
        referenced_geometry: &gdtf::values::Name,
        geometry_address_offset: i32,
    ) -> HashMap<FixtureChannel3Attribute, FixtureChannelFunction> {
        // Find DMX channels that belong to the referenced geometry.
        let dmx_channels_with_geometry = self
            .gdtf_dmx_mode
            .dmx_channels
            .iter()
            .enumerate()
            .filter(|(_, dmx_channel)| dmx_channel.geometry == *referenced_geometry);

        let mut channel_functions = HashMap::new();

        for (c_ix, dmx_channel) in dmx_channels_with_geometry {
            let initial_cf = dmx_channel.initial_function().map(|(_, cf)| cf);

            for (lc_ix, logical_channel) in dmx_channel.logical_channels.iter().enumerate() {
                // NOTE: filter out channel functions with a `NoFeature` attribute as they
                //       interfere with computing DMX ranges.
                let filtered_channel_functions = logical_channel
                    .channel_functions
                    .iter()
                    .filter(|cf| {
                        cf.attribute(&self.gdtf_fixture_type).is_some_and(|a| {
                            a.name.as_ref().is_some_and(|name| &**name != "NoFeature")
                        })
                    })
                    .enumerate()
                    .collect::<Vec<_>>();

                for (cf_ix, channel_function) in &filtered_channel_functions {
                    // Compute the DMX range for this logical function: from current `dmx_from`
                    // up to (but not including) the next function's `dmx_from`, or to max.
                    let from = channel_function.dmx_from.into();
                    let to = filtered_channel_functions
                        .get(cf_ix + 1)
                        .map(|(_, cf)| ClampedValue::from(cf.dmx_from))
                        .unwrap_or_else(|| ClampedValue::new(ClampedValue::MAX));

                    let Some((activation_group, mut attribute)) =
                        self.attribute_from_cf(channel_function)
                    else {
                        continue;
                    };

                    let cf_id = ChannelFunctionId {
                        fixture_path: path,
                        geometry: geometry.clone(),
                        channel_ix: c_ix,
                        logical_channel_ix: lc_ix,
                        channel_function_ix: *cf_ix,
                    };

                    // Determine whether this channel function is physical (has offsets) or virtual.
                    let kind = self.make_channel_function_kind(
                        dmx_channel,
                        &attribute,
                        cf_id.clone(),
                        geometry_address_offset,
                    );

                    let default = ClampedValue::from(channel_function.default);

                    /*
                    // Collect the default values for the initial function.
                    if dmx_channel
                        .initial_function()
                        .is_some_and(|(_, cf)| cf == *channel_function)
                    {
                        match &kind {
                            FixtureChannelFunctionKind::Physical { addresses } => {
                                // let default_values = default.to_address_values(addresses);
                                // self.defaults.extend(default_values);
                            }
                            FixtureChannelFunctionKind::Virtual { .. } => {}
                        }
                    }
                    */

                    let sets = channel_function
                        .channel_sets
                        .iter()
                        .filter_map(|set| {
                            let name = set.name.as_ref().map(|name| name.as_ref().to_string())?;
                            if name.is_empty() {
                                None
                            } else {
                                Some((name, ClampedValue::from(set.dmx_from)))
                            }
                        })
                        .collect();

                    // Some fixtures define multiple channel functions with the same attribute
                    // (i.e. Cameo EVOS W3 Control1)
                    // TODO: find a better way to handle this (maybe sub channel functions, or multiple cf's per attribute)
                    if channel_functions.contains_key(&attribute) {
                        let Some(inc_attribute) = attribute.try_increment() else {
                            continue;
                        };
                        attribute = inc_attribute;
                    }

                    let highlight = dmx_channel.highlight.map(|value| {
                        FixtureChannelFunction::project_cf_value(value.into(), from, to)
                    });

                    channel_functions.insert(
                        attribute,
                        FixtureChannelFunction {
                            kind,
                            min: from,
                            max: to,
                            default,
                            highlight,
                            sets,
                            activation_group,
                            master: logical_channel.master,
                            snap: logical_channel.snap,
                            is_initial: initial_cf
                                .is_some_and(|initial_cf| initial_cf == *channel_function),
                        },
                    );

                    // Record where this channel function was created for relation lookup later.
                    self.channel_function_map.insert(cf_id, path);
                }
            }
        }

        channel_functions
    }

    fn make_channel_function_kind(
        &mut self,
        dmx_channel: &gdtf::dmx_mode::DmxChannel,
        attribute: &FixtureChannel3Attribute,
        cf_id: ChannelFunctionId,
        geometry_address_offset: i32,
    ) -> FixtureChannelFunctionKind {
        match &dmx_channel.offset {
            Some(offsets) => {
                // Physical channel: map each offset to an absolute DMX address.
                let addresses = offsets
                    .iter()
                    .map(|o| {
                        self.address
                            .with_channel_offset(geometry_address_offset + o - 1)
                            .unwrap()
                    })
                    .collect();

                FixtureChannelFunctionKind::Physical { addresses }
            }
            None => {
                // Virtual channel: register for resolution later and return an empty relation set.
                self.register_virtual_channel(attribute.clone(), cf_id);
                FixtureChannelFunctionKind::Virtual { relations: vec![] }
            }
        }
    }

    fn register_virtual_channel(
        &mut self,
        attribute: FixtureChannel3Attribute,
        cf_id: ChannelFunctionId,
    ) {
        self.unresolved_virtual_channels.push((cf_id, attribute));
    }

    fn resolve_virtual_channels(&mut self) {
        // Iterate over virtual channels we registered during the first pass and populate
        // their relation lists by inspecting the DMX mode relations and mapping them to
        // fixtures in our constructed tree.
        for (cf_id, virtual_attribute) in &self.unresolved_virtual_channels {
            let Some(dmx_channel) = self.gdtf_dmx_mode.dmx_channels.get(cf_id.channel_ix) else {
                continue;
            };

            let relations = self.get_relations_for_dmx_channel(&cf_id.geometry, dmx_channel);

            let Some(fixture) = self
                .fixtures
                .iter_mut()
                .find(|f| f.path() == cf_id.fixture_path)
            else {
                continue;
            };

            let Some(virtual_channel_function) =
                fixture.channel_functions.get_mut(&virtual_attribute)
            else {
                continue;
            };

            // Replace the empty relation vector with the resolved relations.
            virtual_channel_function.kind = FixtureChannelFunctionKind::Virtual { relations };
        }
    }

    /// Build relation structures for the provided DMX channel by inspecting DMX mode relations.
    fn get_relations_for_dmx_channel(
        &self,
        master_geometry: &gdtf::values::Name,
        dmx_channel: &gdtf::dmx_mode::DmxChannel,
    ) -> Vec<Relation> {
        let mut channel_relations = Vec::new();

        let relations = self.gdtf_dmx_mode.relations.iter().filter(|relation| {
            relation
                .master(&self.gdtf_dmx_mode)
                .is_some_and(|master| master.name() == dmx_channel.name())
        });

        for relation in relations {
            let Some((dmx_channel, _, follower_channel_function)) =
                relation.follower(&self.gdtf_dmx_mode)
            else {
                log::warn!(
                    "could not find follower for relation with master {}",
                    dmx_channel.name()
                );
                continue;
            };

            let kind = match relation.type_ {
                RelationType::Multiply => RelationKind::Multiply,
                RelationType::Override => RelationKind::Override,
            };

            let Some(geometry) = dmx_channel.geometry(self.gdtf_fixture_type) else {
                log::warn!(
                    "could not find geometry for DMX channel {}",
                    dmx_channel.name()
                );
                continue;
            };

            let geometry_name = match geometry {
                gdtf::geometry::Geometry::Beam(_) => master_geometry,
                _ => geometry.name().unwrap(),
            };

            let Some(fixture_path) =
                self.fixture_path_for_channel_function(&geometry_name, follower_channel_function)
            else {
                log::warn!(
                    "could not find fixture path for follower channel function {}",
                    follower_channel_function
                        .name
                        .as_deref()
                        .unwrap_or("<no name>")
                );
                continue;
            };

            let Some((_, attribute)) = self.attribute_from_cf(follower_channel_function) else {
                continue;
            };

            channel_relations.push(Relation::new(kind, fixture_path, attribute));
        }

        channel_relations
    }

    /// Find the `FixturePath` that corresponds to the provided channel function pointer
    /// and geometry. We need pointer equality because the same `ChannelFunction` instances
    /// (from the DMX mode) are referenced in relations.
    fn fixture_path_for_channel_function(
        &self,
        geometry: &gdtf::values::Name,
        target_channel_function: &gdtf::dmx_mode::ChannelFunction,
    ) -> Option<FixturePath> {
        for (c_ix, dmx_channel) in self.gdtf_dmx_mode.dmx_channels.iter().enumerate() {
            for (lc_ix, logical_channel) in dmx_channel.logical_channels.iter().enumerate() {
                for (cf_ix, channel_function) in
                    logical_channel.channel_functions.iter().enumerate()
                {
                    if !std::ptr::eq(target_channel_function, channel_function) {
                        continue;
                    }

                    // Look up the recorded fixture path for the matching channel function id.
                    if let Some((_, fixture_path)) =
                        self.channel_function_map.iter().find(|(id, _)| {
                            &id.geometry == geometry
                                && id.channel_ix == c_ix
                                && id.logical_channel_ix == lc_ix
                                && id.channel_function_ix == cf_ix
                        })
                    {
                        return Some(*fixture_path);
                    }
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ChannelFunctionId {
    fixture_path: FixturePath,
    geometry: gdtf::values::Name,
    channel_ix: usize,
    logical_channel_ix: usize,
    channel_function_ix: usize,
}

impl From<gdtf::values::DmxValue> for ClampedValue {
    fn from(value: gdtf::values::DmxValue) -> Self {
        let len: u8 = value.bytes().into();
        let raw = value.to(len);
        let max_value = 2_u64.saturating_pow(len as u32 * 8) - 1;
        let floating_value = raw as f32 / max_value as f32;
        ClampedValue::new(floating_value)
    }
}
