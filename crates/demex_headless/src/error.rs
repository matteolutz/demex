#[derive(Debug)]
pub enum DemexHeadlessError {
    FailedToConnect(String, std::io::Error),
    FailedToBindUpdSocket(String, std::io::Error),
    IOError(std::io::Error),
    InvalidPacket,
    InvalidPacketHeader,
    UnknownPacketType(u8),
}

impl std::fmt::Display for DemexHeadlessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DemexHeadlessError::FailedToConnect(ip, err) => {
                write!(f, "Failed to connect to {}: {}", ip, err)
            }
            DemexHeadlessError::FailedToBindUpdSocket(ip, err) => {
                write!(f, "Failed to bind UDP socket to {}: {}", ip, err)
            }
            DemexHeadlessError::IOError(err) => write!(f, "IO error: {}", err),
            DemexHeadlessError::InvalidPacket => write!(f, "Invalid packet received"),
            DemexHeadlessError::InvalidPacketHeader => write!(f, "Invalid packet header"),
            DemexHeadlessError::UnknownPacketType(packet_type) => {
                write!(f, "Unknown packet type: {}", packet_type)
            }
        }
    }
}

impl std::error::Error for DemexHeadlessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
