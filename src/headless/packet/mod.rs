use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

pub mod controller;
pub mod node;

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
