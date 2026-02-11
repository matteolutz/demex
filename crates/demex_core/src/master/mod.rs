use std::collections::HashMap;

use crate::{channel3::clamped_value::ClampedValue, fixture::FixturePath, presets::PresetHandler};

#[derive(Debug, Copy, Clone)]
pub enum SubmasterType {
    Group(u32),
}

#[derive(Debug, Clone)]
pub struct MasterHandler {
    grand_master: ClampedValue,

    group_master_value: HashMap<u32, ClampedValue>,

    cached_fixture_submasters: HashMap<FixturePath, Vec<SubmasterType>>,
}

impl MasterHandler {
    pub fn new(preset_handler: &PresetHandler) -> Self {
        let mut this = Self {
            grand_master: (1.0).into(),
            group_master_value: HashMap::new(),
            cached_fixture_submasters: HashMap::new(),
        };

        this.invalidate_cache(preset_handler);

        this
    }

    pub fn invalidate_cache(&mut self, preset_handler: &PresetHandler) {
        self.cached_fixture_submasters.clear();

        // Group masters
        for group_id in self.group_master_value.keys().copied() {
            let Ok(group) = preset_handler.get_group(group_id) else {
                continue;
            };

            for fixture_path in group.fixture_selection().fixtures() {
                self.cached_fixture_submasters
                    .entry(*fixture_path)
                    .or_default()
                    .push(SubmasterType::Group(group_id));
            }
        }
    }

    pub fn grand_master(&self) -> ClampedValue {
        self.grand_master
    }

    pub fn set_grand_master(&mut self, value: impl Into<ClampedValue>) {
        self.grand_master = value.into();
    }

    pub fn groupmaster_value(&self, group_id: u32) -> Option<ClampedValue> {
        self.group_master_value.get(&group_id).copied()
    }

    pub fn set_groupmaster_value(
        &mut self,
        group_id: u32,
        value: impl Into<ClampedValue>,
        preset_handler: &PresetHandler,
    ) {
        if self
            .group_master_value
            .insert(group_id, value.into())
            .is_none()
        {
            // this means, the value was not present before
            // so we should invalidate the cache
            self.invalidate_cache(preset_handler);
        }
    }

    pub fn get_fixture_master_value(&self, fixture_path: &FixturePath) -> ClampedValue {
        let mut master_value = self.grand_master;

        if let Some(submasters) = self.cached_fixture_submasters.get(fixture_path) {
            for submaster in submasters {
                let submaster_value = match submaster {
                    &SubmasterType::Group(group_id) => self.groupmaster_value(group_id),
                };

                master_value *= submaster_value.unwrap_or(1.0.into());
            }
        }

        master_value
    }
}
