use std::thread::JoinHandle;

use crate::{
    engine::component::ComponentHandle,
    utils::thread::{DemexThreadStatsHandler, demex_update_thread},
};

pub fn start_demex_debug_thread(stats: ComponentHandle<DemexThreadStatsHandler>) -> JoinHandle<()> {
    demex_update_thread("demex-debug".to_owned(), stats.clone(), 1.0, move |_, _| {
        log::debug!("Thread stats:\n{}", stats.read(|stats| stats.to_string()));
    })
}
