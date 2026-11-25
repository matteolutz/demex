use std::{collections::HashMap, hash::Hash};

use itertools::Itertools;

mod global;
pub use global::*;

pub struct DemexUiProfiler<T> {
    max_times: HashMap<T, std::time::Duration>,
    started_measurements: HashMap<T, std::time::Instant>,

    enabled: bool,
}

impl<T> Default for DemexUiProfiler<T> {
    fn default() -> Self {
        Self {
            max_times: HashMap::new(),
            started_measurements: HashMap::new(),
            enabled: true,
        }
    }
}

impl<T> DemexUiProfiler<T> {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl<T> DemexUiProfiler<T>
where
    T: Eq + Hash,
{
    #[inline]
    #[cfg(debug_assertions)]
    pub fn start(&mut self, key: T) {
        if !self.enabled {
            return;
        }

        self.started_measurements
            .insert(key, std::time::Instant::now());
    }

    #[inline]
    #[cfg(not(debug_assertions))]
    pub fn start(&mut self, _key: T) {}

    #[inline]
    #[cfg(debug_assertions)]
    pub fn end(&mut self, key: T) {
        if !self.enabled {
            return;
        }

        let Some(start) = self.started_measurements.remove(&key) else {
            return;
        };

        let elapsed = start.elapsed();

        let current_max = self.max_times.entry(key).or_default();
        *current_max = std::time::Duration::max(*current_max, elapsed);
    }

    #[inline]
    #[cfg(not(debug_assertions))]
    pub fn end(&mut self, _key: T) {}
}

impl<T> std::fmt::Display for DemexUiProfiler<T>
where
    T: Eq + Hash + std::fmt::Debug + Ord,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, max_time) in self.max_times.iter().sorted_by_key(|(key, _)| *key) {
            writeln!(f, "{:?}: {:#?}", key, max_time)?;
        }
        Ok(())
    }
}
