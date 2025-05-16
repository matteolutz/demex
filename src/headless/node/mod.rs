use std::{net, time};

use crate::{
    headless::{
        packet::{controller::DemexProtoControllerPacket, headless::DemexProtoHeadlessPacket},
        protocol::Protocol,
        sync::DemexProtoSync,
        DEMEX_HEADLESS_TCP_PORT,
    },
    show::{context::ShowContext, DemexNoUiShow},
    utils::version::VERSION_STR,
};

use super::error::DemexHeadlessError;

pub struct DemexHeadlessNode {}

impl DemexHeadlessNode {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start_headless_in_current_thread(
        &mut self,
        master_ip: String,
        mut show_context: ShowContext,
    ) -> Result<(), DemexHeadlessError> {
        log::debug!("Connecting to {}..", master_ip);

        let tcp_stream = net::TcpStream::connect((master_ip.as_str(), DEMEX_HEADLESS_TCP_PORT))
            .map_err(|err| DemexHeadlessError::FailedToConnect(master_ip, err))?;

        let mut proto = Protocol::with_stream(tcp_stream).map_err(DemexHeadlessError::IOError)?;
        let mut last_sync: Option<time::Instant> = None;

        log::debug!("Connected, waiting for packets by the controller..");

        loop {
            if last_sync.is_some_and(|last_sync| last_sync.elapsed().as_secs() > 5) {
                let _ = proto.send_packet(&DemexProtoHeadlessPacket::SyncRequest);
                // prevent request sync two times, if other packets are still to be read
                last_sync = None;
            }

            let packet = proto.read_packet::<DemexProtoControllerPacket>();

            if let Ok(packet) = packet {
                log::debug!("Received DemexProto packet: {:?}", packet);

                match packet {
                    DemexProtoControllerPacket::HeadlessInfoRequest => {
                        let _ =
                            proto.send_packet(&DemexProtoHeadlessPacket::HeadlessInfoResponse {
                                version: VERSION_STR.to_owned(),
                            });
                    }
                    DemexProtoControllerPacket::ShowFileUpdate => {
                        let _ = proto.send_packet(&DemexProtoHeadlessPacket::ShowFileRequest);
                    }
                    DemexProtoControllerPacket::ShowFile { show_file } => {
                        if let Ok(show) = serde_json::from_slice::<DemexNoUiShow>(&show_file) {
                            log::debug!("Received show file, updating..");
                            show_context.update_from(show, true);
                        }
                    }
                    DemexProtoControllerPacket::Sync { sync } => {
                        if let Ok(sync) = serde_json::from_slice::<DemexProtoSync>(&sync) {
                            log::debug!("Received sync, applying..");
                            sync.apply(&show_context);
                            last_sync = Some(time::Instant::now());
                        }
                    }
                }
            }
        }
    }
}
