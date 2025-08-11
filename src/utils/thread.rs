use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle, ThreadId},
    time,
};

use parking_lot::RwLock;

pub fn demex_simple_thread<F: Fn(Arc<RwLock<DemexThreadStatsHandler>>, &str) + Send + 'static>(
    name: String,
    stats: Arc<RwLock<DemexThreadStatsHandler>>,
    f: F,
) -> JoinHandle<()> {
    let stats_cloned = stats.clone();
    let name_cloned = name.to_owned();

    let handle = thread::spawn(move || {
        f(stats, name.as_str());
    });

    stats_cloned
        .write()
        .register_thread(name_cloned, handle.thread().id());

    handle
}

pub fn demex_update_thread<F: Fn(f64, &mut time::Instant) + Send + 'static>(
    name: String,
    stats: Arc<RwLock<DemexThreadStatsHandler>>,
    fps: f64,
    f: F,
) -> JoinHandle<()> {
    let stats_cloned = stats.clone();
    let name_cloned = name.to_owned();

    let mut last_user_update = time::Instant::now();

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

            f(delta_time, &mut last_user_update);

            stats.write().update(name.as_str(), delta_time);
        }
    });

    stats_cloned
        .write()
        .register_thread(name_cloned, handle.thread().id());

    handle
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
        self.name_to_id.insert(name.clone(), id);
        self.stats.insert(name, DemexThreadStats::new(0.0));
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
