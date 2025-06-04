use std::{net, time};

use crate::{
    headless::{
        packet::{controller::DemexProtoControllerPacket, node::DemexProtoHeadlessNodePacket},
        protocol::Protocol,
        DEMEX_HEADLESS_TCP_PORT,
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
            .map_err(|err| DemexHeadlessError::FailedToConnect(master_ip, err))?;

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
                    DemexProtoControllerPacket::Action { action: _ } => {
                        log::debug!("Received action, executing..");
                    }
                }
            }
        }
    }
}
