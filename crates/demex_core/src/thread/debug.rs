use crate::thread::DemexThreadDelegate;

#[derive(Default)]
pub struct DebugThread {}

impl DemexThreadDelegate for DebugThread {
    type ThreadMessage = ();

    fn name() -> &'static str {
        "demex-debug"
    }

    fn its() -> f64 {
        1.0
    }

    fn update(&mut self, thread: &mut super::DemexThread<Self>) -> bool {
        for _ in thread.handle_messages() {}
        if thread.should_stop() {
            return true;
        }

        log::debug!(
            "Thread stats:\n{}",
            thread.stats().read(|stats| stats.to_string())
        );

        false
    }
}
