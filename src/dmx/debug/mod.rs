use std::{
    sync::mpsc::{self, TryRecvError},
    thread,
};

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use super::DmxData;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, EguiProbe, Serialize, Deserialize)]
pub enum DebugOutputVerbosity {
    Verbose,
    Quiet,

    #[default]
    Silent,
}

pub fn start_debug_output_thread(
    rx: mpsc::Receiver<DmxData>,
    output_verbosity: DebugOutputVerbosity,
) {
    thread::spawn(move || loop {
        let recv_result = rx.try_recv();

        if let Ok((universe, universe_data)) = recv_result {
            match output_verbosity {
                DebugOutputVerbosity::Verbose => {
                    println!(
                        "Sending data on universe {}:\n{:?}",
                        universe, universe_data
                    );
                }
                DebugOutputVerbosity::Quiet => {
                    println!("Sending data on universe {}", universe);
                }
                DebugOutputVerbosity::Silent => {}
            }
        } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
            break;
        }
    });
}
