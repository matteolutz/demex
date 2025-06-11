use std::{collections::HashMap, net, sync::Arc, thread::JoinHandle, time};

use parking_lot::RwLock;

use crate::{
    fixture::patch::SerializablePatch,
    headless::{sync::DemexProtoSync, DEMEX_HEADLESS_CONTROLLER_UDP_PORT},
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

struct DemexHeadlessNode {
    pub udp_addr: net::SocketAddr,
}

#[derive(Default)]
pub struct DemexHeadlessConroller {
    nodes: Arc<RwLock<HashMap<u32, DemexHeadlessNode>>>,
}

impl DemexHeadlessConroller {
    pub fn start_controller_thread(
        self,
        stats: Arc<RwLock<DemexThreadStatsHandler>>,
        show_context: ShowContext,
    ) -> JoinHandle<()> {
        thread::demex_simple_thread("demex-proto".to_string(), stats, move |_, _| {
            let tcp_listener =
                net::TcpListener::bind(("0.0.0.0", DEMEX_HEADLESS_TCP_PORT)).unwrap();
            let udp_socket =
                net::UdpSocket::bind(("0.0.0.0", DEMEX_HEADLESS_CONTROLLER_UDP_PORT)).unwrap();

            log::debug!("Started TCP listener and created UDP socket for headless controller");

            {
                let _show_context = show_context.clone();
                let nodes = self.nodes.clone();

                std::thread::spawn(move || loop {
                    for (_, node_data) in nodes.read().iter() {
                        let _ = udp_socket.send_to("demex".as_bytes(), node_data.udp_addr);

                        std::thread::sleep(time::Duration::from_secs_f32(1.0 / 30.0));
                    }
                });
            }

            for stream in tcp_listener.incoming() {
                let show_context = show_context.clone();
                let nodes = self.nodes.clone();

                std::thread::spawn(move || {
                    let mut node_state = DemexHeadlessNodeState::default();

                    let mut protocol = Protocol::with_stream(stream.unwrap()).unwrap();

                    let _ = protocol.send_packet(&DemexProtoControllerPacket::HeadlessInfoRequest);

                    loop {
                        let packet = protocol.read_packet::<DemexProtoHeadlessNodePacket>();
                        if let Ok(packet) = packet {
                            log::debug!("Received demex proto packet: {:#x}", u8::from(&packet));

                            match packet {
                                DemexProtoHeadlessNodePacket::HeadlessInfoResponse {
                                    id,
                                    version,
                                    udp_addr,
                                } => {
                                    if version != VERSION_STR {
                                        log::warn!(
                                            "Version mismatch: {} (node) != {} (controller), shutting down..",
                                            version,
                                            VERSION_STR
                                        );
                                        break;
                                    }

                                    if nodes.read().contains_key(&id) {
                                        log::warn!("Duplicate node id: {id}, shutting down..");
                                        break;
                                    }

                                    node_state = DemexHeadlessNodeState::Verified;
                                    nodes.write().insert(id, DemexHeadlessNode { udp_addr });

                                    log::info!("Got new node with udp_addr: {}", udp_addr);

                                    let _ = protocol
                                        .send_packet(&DemexProtoControllerPacket::ShowFileUpdate);
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
