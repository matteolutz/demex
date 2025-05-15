use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

pub mod controller;
pub mod headless;

pub fn demex_proto_write_string(writer: &mut impl Write, str: &str) -> usize {
    let str_bytes = str.as_bytes();
    let len = str_bytes.len() as u16;
    writer.write_u16::<NetworkEndian>(len).unwrap();
    writer.write_all(str_bytes).unwrap();
    len as usize + 2
}

pub fn demex_proto_read_string(reader: &mut impl Read) -> io::Result<String> {
    let len = reader.read_u16::<NetworkEndian>()? as usize;
    let mut buf = vec![0; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))
}

pub trait DemexProtoSerialize {
    fn serialize(&self, buf: &mut impl Write) -> std::io::Result<usize>;
}

pub trait DemexProtoDeserialize {
    type Output;

    fn deserialize(buf: &mut impl Read) -> std::io::Result<Self::Output>;
}
