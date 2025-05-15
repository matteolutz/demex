use std::net;

use error::DemexHeadlessError;
use packet::{controller::DemexProtoControllerPacket, headless::DemexProtoHeadlessPacket};
use protocol::Protocol;

use crate::utils::version::VERSION_STR;

pub mod error;
pub mod packet;
pub mod protocol;

const DEMEX_HEADLESS_TCP_PORT: u16 = 4546;

pub struct DemexHeadless {}

impl DemexHeadless {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start_headless_in_current_thread(
        &mut self,
        master_ip: String,
    ) -> Result<(), DemexHeadlessError> {
        log::debug!("Connecting to {}..", master_ip);

        let tcp_stream = net::TcpStream::connect((master_ip.as_str(), DEMEX_HEADLESS_TCP_PORT))
            .map_err(|err| DemexHeadlessError::FailedToConnect(master_ip, err))?;

        let mut proto = Protocol::with_stream(tcp_stream).map_err(DemexHeadlessError::IOError)?;

        log::debug!("Connected, waiting for packets by the controller..");

        loop {
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
                }
            }
        }
    }
}
