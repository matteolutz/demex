use std::{
    net::{self},
    time,
};

use crate::{
    headless::{
        packet::{
            controller::DemexProtoControllerPacket, controller_udp::DemexProtoUdpControllerPacket,
            node::DemexProtoHeadlessNodePacket, DemexProtoDeserialize,
        },
        protocol::Protocol,
        DEMEX_HEADLESS_CONTROLLER_UDP_PORT, DEMEX_HEADLESS_NODE_UDP_PORT, DEMEX_HEADLESS_TCP_PORT,
    },
    show::context::ShowContext,
    utils::version::VERSION_STR,
};

use super::{error::DemexHeadlessError, id::DemexProtoDeviceId};

#[derive(Default)]
pub struct DemexHeadlessNode {}

impl DemexHeadlessNode {
    pub fn start_headless_in_current_thread(
        &mut self,
        master_ip: String,
        node_id: u32,
        mut show_context: ShowContext,
    ) -> Result<(), DemexHeadlessError> {
        log::debug!("Connecting to {}..", master_ip);

        let device_id = DemexProtoDeviceId::Node(node_id);

        let tcp_stream = net::TcpStream::connect((master_ip.as_str(), DEMEX_HEADLESS_TCP_PORT))
            .map_err(|err| DemexHeadlessError::FailedToConnect(master_ip.clone(), err))?;

        let udp_socket = net::UdpSocket::bind(("0.0.0.0", DEMEX_HEADLESS_NODE_UDP_PORT))
            .map_err(|err| DemexHeadlessError::FailedToBindUpdSocket(master_ip.clone(), err))?;
        udp_socket
            .connect((master_ip.as_str(), DEMEX_HEADLESS_CONTROLLER_UDP_PORT))
            .map_err(|err| DemexHeadlessError::FailedToBindUpdSocket(master_ip, err))?;

        let udp_addr = udp_socket
            .local_addr()
            .map_err(DemexHeadlessError::IOError)?;

        {
            let fixture_handler = show_context.fixture_handler.clone();

            std::thread::spawn(move || {
                let mut udp_buffer = [0u8; 16_384];

                loop {
                    if let Ok(packet) = udp_socket.recv(&mut udp_buffer).and_then(|bytes_read| {
                        DemexProtoUdpControllerPacket::deserialize(
                            &mut udp_buffer[..bytes_read].as_ref(),
                        )
                    }) {
                        match packet {
                            DemexProtoUdpControllerPacket::FixtureOutputValuesUpdate { values } => {
                                let mut fixture_handler = fixture_handler.write();

                                for (fixture_id, channel_name, value) in values {
                                    if let Some(fixture) = fixture_handler.fixture(fixture_id) {
                                        let _ =
                                            fixture.internal_set_output_value(&channel_name, value);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }

        let mut proto = Protocol::with_stream(tcp_stream).map_err(DemexHeadlessError::IOError)?;
        let mut last_sync: Option<time::Instant> = None;

        log::debug!("Connected, waiting for packets by the controller..");

        loop {
            if last_sync.is_some_and(|last_sync| last_sync.elapsed().as_secs() > 5) {
                let _ = proto.send_packet(&DemexProtoHeadlessNodePacket::SyncRequest);
                // prevent request sync two times, if other packets are still to be read
                last_sync = None;
            }

            let packet = proto.read_packet::<DemexProtoControllerPacket>();

            if let Ok(packet) = packet {
                log::debug!("Received DemexProto packet: {:#x}", u8::from(&packet));

                match packet {
                    DemexProtoControllerPacket::HeadlessInfoRequest => {
                        let _ = proto.send_packet(
                            &DemexProtoHeadlessNodePacket::HeadlessInfoResponse {
                                id: node_id,
                                version: VERSION_STR.to_owned(),
                                udp_addr,
                            },
                        );
                    }
                    DemexProtoControllerPacket::ShowFileUpdate => {
                        let _ = proto.send_packet(&DemexProtoHeadlessNodePacket::ShowFileRequest);
                    }
                    DemexProtoControllerPacket::ShowFile { show_file } => {
                        log::debug!("Received show file, updating..");
                        show_context.update_from(*show_file, device_id);
                    }
                    DemexProtoControllerPacket::Sync { sync } => {
                        log::debug!("Received sync, applying..");
                        sync.apply(&show_context);
                        last_sync = Some(time::Instant::now());
                    }
                }
            }
        }
    }
}
