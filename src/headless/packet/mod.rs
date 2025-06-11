use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

pub mod controller;
pub mod node;

pub fn demex_proto_write_sock_addr(
    writer: &mut impl Write,
    addr: std::net::SocketAddr,
) -> io::Result<usize> {
    let ip = addr.ip();
    let port = addr.port();

    let mut bytes_written = 0;

    writer.write_u16::<NetworkEndian>(port)?;
    bytes_written += 2;

    match ip {
        std::net::IpAddr::V4(ipv4) => {
            writer.write_u8(4)?; // IPv4
            bytes_written += 1;
            writer.write_all(&ipv4.octets())?;
            bytes_written += 4;
        }
        std::net::IpAddr::V6(ipv6) => {
            writer.write_u8(6)?; // IPv6
            bytes_written += 1;
            writer.write_all(&ipv6.octets())?;
            bytes_written += 16;
        }
    }

    Ok(bytes_written)
}

pub fn demex_proto_read_sock_addr(reader: &mut impl Read) -> io::Result<std::net::SocketAddr> {
    let port = reader.read_u16::<NetworkEndian>()? as u16;

    let ip_version = reader.read_u8()?;
    let mut ip_bytes = [0; 16];

    match ip_version {
        4 => {
            reader.read_exact(&mut ip_bytes[..4])?;

            let ip_bytes: [u8; 4] = ip_bytes[..4].try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid IPv4 address length")
            })?;

            Ok(std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::from(ip_bytes)),
                port,
            ))
        }
        6 => {
            reader.read_exact(&mut ip_bytes[..16])?;
            Ok(std::net::SocketAddr::new(
                std::net::IpAddr::V6(std::net::Ipv6Addr::from(ip_bytes)),
                port,
            ))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unsupported IP version",
        )),
    }
}

pub fn demex_proto_write_string(writer: &mut impl Write, str: &str) -> io::Result<usize> {
    let str_bytes = str.as_bytes();
    let len = str_bytes.len() as u16;

    writer.write_u16::<NetworkEndian>(len)?;
    writer.write_all(str_bytes)?;

    Ok(len as usize + 2)
}

pub fn demex_proto_read_string(reader: &mut impl Read) -> io::Result<String> {
    let len = reader.read_u16::<NetworkEndian>()? as usize;
    let mut buf = vec![0; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))
}

pub fn demex_proto_write_u16(writer: &mut impl Write, value: u16) -> io::Result<usize> {
    writer.write_u16::<NetworkEndian>(value)?;
    Ok(2)
}

pub fn demex_proto_read_u16(reader: &mut impl Read) -> io::Result<u16> {
    reader.read_u16::<NetworkEndian>()
}

pub fn demex_proto_write_u32(writer: &mut impl Write, value: u32) -> io::Result<usize> {
    writer.write_u32::<NetworkEndian>(value)?;
    Ok(4)
}

pub fn demex_proto_read_u32(reader: &mut impl Read) -> io::Result<u32> {
    reader.read_u32::<NetworkEndian>()
}

pub fn demex_proto_write_u64(writer: &mut impl Write, value: u64) -> io::Result<usize> {
    writer.write_u64::<NetworkEndian>(value)?;
    Ok(8)
}

pub fn demex_proto_read_u64(reader: &mut impl Read) -> io::Result<u64> {
    reader.read_u64::<NetworkEndian>()
}

pub fn demex_proto_write_bytes(writer: &mut impl Write, bytes: &[u8]) -> io::Result<usize> {
    writer.write_all(bytes)?;
    Ok(bytes.len())
}

pub fn demex_proto_read_bytes(reader: &mut impl Read, bytes: &mut [u8]) -> io::Result<()> {
    reader.read_exact(bytes)
}

pub trait DemexProtoSerialize {
    fn serialize(&self, buf: &mut impl Write) -> std::io::Result<usize>;
}

pub trait DemexProtoDeserialize {
    type Output;

    fn deserialize(buf: &mut impl Read) -> std::io::Result<Self::Output>;
}
