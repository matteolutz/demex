use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, ThreadId},
};

use parking_lot::RwLock;

pub fn demex_thread<F: Fn(f64) + Send + 'static>(
    name: String,
    stats: Arc<RwLock<DemexThreadStatsHandler>>,
    fps: f64,
    f: F,
) {
    let stats_cloned = stats.clone();
    let name_cloned = name.to_owned();

    let handle = thread::spawn(move || {
        let mut last_update = std::time::Instant::now();

        loop {
            let elapsed = last_update.elapsed().as_secs_f64();
            let diff = (1.0 / fps) - elapsed;

            if diff > 0.0 {
                thread::sleep(std::time::Duration::from_secs_f64(diff));
            }

            let real_elapsed = last_update.elapsed();

            let delta_time = real_elapsed.as_secs_f64();

            last_update = std::time::Instant::now();

            f(delta_time);

            stats.write().update(name.as_str(), delta_time);
        }
    });

    stats_cloned
        .write()
        .register_thread(name_cloned, handle.thread().id());
}

#[derive(Debug, Default)]
pub struct DemexThreadStats {
    dt: f64,
    max_dt: f64,
}

impl DemexThreadStats {
    pub fn new(dt: f64) -> Self {
        Self { dt, max_dt: dt }
    }

    pub fn dt(&self) -> f64 {
        self.dt
    }

    pub fn max_dt(&self) -> f64 {
        self.max_dt
    }
}

#[derive(Default)]
pub struct DemexThreadStatsHandler {
    stats: HashMap<String, DemexThreadStats>,
    name_to_id: HashMap<String, ThreadId>,
}

impl DemexThreadStatsHandler {
    pub fn register_thread(&mut self, name: String, id: ThreadId) {
        self.name_to_id.insert(name, id);
    }

    pub fn update(&mut self, name: &str, dt: f64) {
        if let Some(stats) = self.stats.get_mut(name) {
            stats.dt = dt;
            stats.max_dt = stats.max_dt.max(dt);
        } else {
            self.stats
                .insert(name.to_string(), DemexThreadStats::new(dt));
        }
    }

    pub fn thread_id(&self, name: &str) -> Option<ThreadId> {
        self.name_to_id.get(name).copied()
    }

    pub fn stats(&self) -> &HashMap<String, DemexThreadStats> {
        &self.stats
    }
}
