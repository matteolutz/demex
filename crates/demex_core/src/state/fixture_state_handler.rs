use std::{
    collections::{HashMap, HashSet},
    sync::mpsc,
};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3,
        channel_value_queue::ChannelValueQueueEntry,
    },
    engine::component::Component,
    fixture::{
        Fixture, FixtureChannelFunctionInitial, FixturePath, FixturePathMatchLevel,
        error::FixtureError,
    },
    master::MasterHandler,
    patch::Patch,
    presets::PresetHandler,
    selection::FixtureSelection,
    state::fixture_state::FixtureState,
    timing::TimingHandler,
    updatables::UpdatableHandler,
    value_source::FixtureChannelValueSourceTrait,
};

impl Component for FixtureStateHandler {}

#[derive(Debug)]
pub struct FixtureStateHandler {
    fixture_states: HashMap<FixturePath, FixtureState>,
}

impl Default for FixtureStateHandler {
    fn default() -> Self {
        Self {
            fixture_states: Default::default(),
        }
    }
}

impl FixtureStateHandler {
    pub fn new<'a>(fixtures: impl IntoIterator<Item = &'a Fixture>) -> Result<Self, FixtureError> {
        Ok(Self {
            fixture_states: fixtures
                .into_iter()
                .map(|f| (f.path(), FixtureState::new(f)))
                .collect(),
        })
    }

    /// Set default fixture states for fixtures that are present in the patch
    /// but not in the current fixture states.
    pub fn insert_new_fixtures(&mut self, patch: &Patch) {
        for fixture in patch.fixtures() {
            if self.fixture_states.contains_key(&fixture.path) {
                continue;
            }

            self.fixture_states
                .insert(fixture.path, FixtureState::new(fixture));
        }
    }

    pub fn fixtures(&self) -> &HashMap<FixturePath, FixtureState> {
        &self.fixture_states
    }

    pub fn fixture(&self, fixture_path: &FixturePath) -> Result<&FixtureState, FixtureError> {
        self.fixture_states
            .get(fixture_path)
            .ok_or(FixtureError::NotFound(*fixture_path))
    }

    pub fn fixture_mut(
        &mut self,
        fixture_path: &FixturePath,
    ) -> Result<&mut FixtureState, FixtureError> {
        self.fixture_states
            .get_mut(fixture_path)
            .ok_or(FixtureError::NotFound(*fixture_path))
    }

    pub fn home_all(&mut self, clear_sources: bool) -> Result<(), FixtureError> {
        for (_, state) in self.fixture_states.iter_mut() {
            state.home(clear_sources)?;
        }

        Ok(())
    }

    pub fn update_output_values(
        &mut self,
        patch: &Patch,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
        highlight: Option<&FixtureSelection>,
        updated_output_values: &mut HashMap<
            FixturePath,
            HashMap<FixtureChannel3Attribute, FixtureChannelValue3>,
        >,
    ) -> Result<(), FixtureError> {
        for (path, state) in self.fixture_states.iter_mut() {
            let fixture = patch.fixture(path)?;

            for (attribute, cf) in fixture.channel_functions() {
                let mut new_output_value = state.sources().get_attribute_value(
                    &fixture.path,
                    state,
                    attribute,
                    updatable_handler,
                    preset_handler,
                    timing_handler,
                )?;

                if highlight.is_some_and(|highlight| {
                    // all child fixtures should also be highlighted
                    highlight.has_fixture_with_level(path, FixturePathMatchLevel::NotSibling)
                }) {
                    if let Some(highlight) = cf.highlight() {
                        new_output_value = FixtureChannelValue3::discrete(highlight);
                    }
                }

                let output_value = state.cached_output_mut().get_mut(attribute).unwrap();

                if !output_value.force_update && output_value.value == new_output_value {
                    continue;
                }

                output_value.update(new_output_value.clone());
                updated_output_values
                    .entry(*path)
                    .or_default()
                    .insert(*attribute, new_output_value.clone());
            }
        }

        Ok(())
    }

    // TODO: optimize this really, really bad function
    pub fn submit_output_values(
        &mut self,
        value_queue_tx: &mpsc::Sender<ChannelValueQueueEntry>,
        patch: &Patch,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        master_handler: &mut MasterHandler,
    ) -> Result<(), FixtureError> {
        // this contains attributes, that we need to force update
        // in a second pass
        let mut force_output_attributes = HashSet::new();

        for (path, state) in self.fixture_states.iter_mut() {
            force_output_attributes.clear();
            let mut updated_values = HashMap::new();

            let fixture = patch.fixture(path)?;

            // is this fixture affected by any master changes?
            let output_for_master_change = master_handler.force_output().should_force_output(path);

            for (attribute, output_value) in state.cached_output_mut() {
                let cf = fixture.channel_function(attribute);

                // is this attribute affected by master changes or is it in the force output set?
                let should_force_output_attribute = (output_for_master_change
                    && cf.as_ref().is_some_and(|cf| cf.should_react_to_master()))
                    || force_output_attributes.contains(attribute);

                if !should_force_output_attribute && !output_value.should_output(preset_handler) {
                    continue;
                }

                // if we added this attribute to the force output set in a previous iteration,
                // remove it now that we are processing it
                force_output_attributes.remove(attribute);

                // get the discrete value for this attribute
                let discrete_value = output_value.value().clone().to_discrete(
                    fixture,
                    &attribute,
                    preset_handler,
                    timing_handler,
                );

                // insert the updated value into the map
                updated_values.insert(*attribute, (discrete_value, None));

                // this means we just homed this attribute
                if output_value.value().is_home()
                // if the channel function is not initial, we should also output the initial
                // attribute
                    && let Some(FixtureChannelFunctionInitial::Other(other)) = cf.map(|cf| cf.initial)
                {
                    log::debug!("force outputting initial attribute: {}", other);
                    // add the initial attribute to the force output set
                    force_output_attributes.insert(other.clone());
                }

                // reset the output value to clear any pending updates
                output_value.reset();
            }

            // add any remaining force output attributes to the queue
            for force_output_attribute in force_output_attributes.drain() {
                let Some(output_value) = state.cached_output_mut().get_mut(&force_output_attribute)
                else {
                    continue;
                };

                let discrete_value = output_value.value().clone().to_discrete(
                    fixture,
                    &force_output_attribute,
                    preset_handler,
                    timing_handler,
                );

                updated_values.insert(force_output_attribute, (discrete_value, None));

                // the output_value should already be reset.
                // If it was flagged dirty, it should have been cleared by the previous pass.
                // But just to make sure..
                output_value.reset();
            }

            if !updated_values.is_empty() {
                let master_value = master_handler.get_fixture_master_value(path);

                value_queue_tx
                    .send(ChannelValueQueueEntry {
                        fixture_path: *path,
                        master_value,
                        values: updated_values,
                    })
                    .expect("Output channel has hung up");
            }
        }

        master_handler.update();

        Ok(())
    }
}
