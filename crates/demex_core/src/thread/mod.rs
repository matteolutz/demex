use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
    time,
};

use crate::{engine::component::ComponentHandle, utils::thread::DemexThreadStatsHandler};
pub mod debug;
pub mod output;
pub mod update;

pub trait DemexThreadDelegate: Send + Sized + 'static {
    type ThreadMessage: Send;

    /// Unique identifier for the thread.
    fn name() -> &'static str;

    /// Iterations per second.
    fn its() -> f64;

    /// Update the thread. Returns true if the thread should stop.
    fn update(&mut self, thread: &mut DemexThread<Self>) -> bool;
}

pub struct DemexThreadHandle<D: DemexThreadDelegate> {
    join_handle: JoinHandle<()>,
    tx: mpsc::Sender<DemexThreadMessage<D>>,
}

impl<D: DemexThreadDelegate> DemexThreadHandle<D> {
    pub fn join(self) -> thread::Result<()> {
        self.join_handle.join()
    }

    pub fn stop(&self) -> Result<(), mpsc::SendError<DemexThreadMessage<D>>> {
        self.tx.send(DemexThreadMessage::Stop)
    }

    pub fn stop_and_join(self) -> thread::Result<()> {
        self.stop().expect("Should stop the thread");
        self.join()
    }

    pub fn send(
        &self,
        message: D::ThreadMessage,
    ) -> Result<(), mpsc::SendError<DemexThreadMessage<D>>> {
        self.tx.send(DemexThreadMessage::Update(message))
    }

    pub fn sender(&self) -> mpsc::Sender<DemexThreadMessage<D>> {
        self.tx.clone()
    }
}

pub enum DemexThreadMessage<D: DemexThreadDelegate> {
    Update(D::ThreadMessage),
    Stop,
}

pub struct DemexThread<D: DemexThreadDelegate> {
    rx: mpsc::Receiver<DemexThreadMessage<D>>,
    last_user_update: time::Instant,

    stats: ComponentHandle<DemexThreadStatsHandler>,

    should_stop: bool,
}

impl<D: DemexThreadDelegate> DemexThread<D> {
    pub fn last_user_update(&mut self) -> &mut time::Instant {
        &mut self.last_user_update
    }

    pub fn should_stop(&self) -> bool {
        self.should_stop
    }

    pub fn stats(&self) -> &ComponentHandle<DemexThreadStatsHandler> {
        &self.stats
    }

    pub fn handle_messages(&mut self) -> impl Iterator<Item = D::ThreadMessage> {
        // kinda ugly with those side effects, but it works
        self.rx.try_iter().filter_map(|message| match message {
            DemexThreadMessage::Stop => {
                self.should_stop = true;
                None
            }
            DemexThreadMessage::Update(update) => Some(update),
        })
    }
}

impl<D: DemexThreadDelegate> DemexThread<D> {
    pub fn start(
        mut delegate: D,
        mut stats: ComponentHandle<DemexThreadStatsHandler>,
    ) -> DemexThreadHandle<D> {
        let (tx, rx) = mpsc::channel();

        let mut stats_cloned = stats.clone();
        let mut thread = DemexThread {
            rx,
            last_user_update: time::Instant::now(),
            stats: stats.clone(),
            should_stop: false,
        };

        let join_handle = thread::spawn(move || {
            let mut last_update = std::time::Instant::now();

            loop {
                let elapsed = last_update.elapsed().as_secs_f64();
                let diff = (1.0 / D::its()) - elapsed;

                if diff > 0.0 {
                    thread::sleep(std::time::Duration::from_secs_f64(diff));
                }

                let real_elapsed = last_update.elapsed();

                let delta_time = real_elapsed.as_secs_f64();

                last_update = std::time::Instant::now();

                let stop_thread = delegate.update(&mut thread);

                stats.write(|stats| stats.update(D::name(), delta_time));

                if stop_thread {
                    break;
                }
            }
        });

        stats_cloned
            .write(|stats| stats.register_thread(D::name().to_string(), join_handle.thread().id()));

        DemexThreadHandle { join_handle, tx }
    }
}
