use std::{net, sync::Arc, thread::JoinHandle};

use parking_lot::RwLock;

use crate::{
    fixture::patch::SerializablePatch,
    headless::sync::DemexProtoSync,
    show::{context::ShowContext, DemexNoUiShow},
    utils::{
        thread::{self, DemexThreadStatsHandler},
        version::VERSION_STR,
    },
};

use super::{
    packet::{controller::DemexProtoControllerPacket, node::DemexProtoHeadlessNodePacket},
    protocol::Protocol,
    DEMEX_HEADLESS_TCP_PORT,
};

#[derive(Default, Debug, PartialEq, Eq)]
enum DemexHeadlessNodeState {
    #[default]
    NotVerified,

    Verified,
}

pub struct DemexHeadlessConroller {}

impl DemexHeadlessConroller {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start_controller_thread(
        &self,
        stats: Arc<RwLock<DemexThreadStatsHandler>>,
        show_context: ShowContext,
    ) -> JoinHandle<()> {
        thread::demex_simple_thread("demex-proto".to_string(), stats, move |_, _| {
            let listener = net::TcpListener::bind(("0.0.0.0", DEMEX_HEADLESS_TCP_PORT)).unwrap();
            log::debug!("Started listener");

            for stream in listener.incoming() {
                let show_context = show_context.clone();

                std::thread::spawn(move || {
                    let mut node_state = DemexHeadlessNodeState::default();

                    let mut protocol = Protocol::with_stream(stream.unwrap()).unwrap();

                    let _ = protocol.send_packet(&DemexProtoControllerPacket::HeadlessInfoRequest);

                    loop {
                        let packet = protocol.read_packet::<DemexProtoHeadlessNodePacket>();
                        if let Ok(packet) = packet {
                            log::debug!("Received demex proto packet: {:#x}", u8::from(&packet));

                            match packet {
                                DemexProtoHeadlessNodePacket::HeadlessInfoResponse { version } => {
                                    if version != VERSION_STR {
                                        break;
                                    }

                                    let _ = protocol
                                        .send_packet(&DemexProtoControllerPacket::ShowFileUpdate);
                                    node_state = DemexHeadlessNodeState::Verified;
                                }
                                DemexProtoHeadlessNodePacket::ShowFileRequest => {
                                    if node_state != DemexHeadlessNodeState::Verified {
                                        break;
                                    }

                                    let show_file = Box::new(DemexNoUiShow {
                                        preset_handler: show_context.preset_handler.read().clone(),
                                        updatable_handler: show_context
                                            .updatable_handler
                                            .read()
                                            .clone(),
                                        timing_handler: show_context.timing_handler.read().clone(),
                                        patch: SerializablePatch::from_patch(
                                            &show_context.patch.read(),
                                        ),
                                    });

                                    protocol
                                        .send_packet(&DemexProtoControllerPacket::ShowFile {
                                            show_file,
                                        })
                                        .unwrap();

                                    protocol
                                        .send_packet(&DemexProtoControllerPacket::Sync {
                                            sync: Box::new(DemexProtoSync::get(&show_context)),
                                        })
                                        .unwrap();
                                }
                                DemexProtoHeadlessNodePacket::SyncRequest => {
                                    if node_state != DemexHeadlessNodeState::Verified {
                                        break;
                                    }

                                    protocol
                                        .send_packet(&DemexProtoControllerPacket::Sync {
                                            sync: Box::new(DemexProtoSync::get(&show_context)),
                                        })
                                        .unwrap();
                                }
                            }
                        }
                    }

                    let _ = protocol.shutdown(net::Shutdown::Both);
                });
            }
        })
    }
}
