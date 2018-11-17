use byteorder::{ByteOrder, NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::collections::VecDeque;
use std::io::{Cursor, Write};

/// [PacketReader] represents a recieved data packet and allows for extracting components in order.
pub struct PacketReader(VecDeque<u8>);

impl PacketReader {
    /// Creates a [Packet] from the given [Vec<u8>].
    pub fn from_vec(vec: Vec<u8>) -> Self {
        PacketReader(VecDeque::from(vec))
    }

    /// Returns a [Vec] of the remaining bytes.
    pub fn into_vec(self) -> Vec<u8> {
        Vec::from(self.0)
    }

    /// Returns the number of bytes left in the [Packet].
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an [Option] containing the next byte if there is one or [None] otherwise.
    pub fn next_u8(&mut self) -> Option<u8> {
        self.0.pop_front()
    }

    /// Parses and returns the next two bytes as a [u16] if they are present, otherwaise returns [None].
    pub fn next_u16(&mut self) -> Option<u16> {
        if self.0.len() < 2 {
            None
        } else {
            Some(NetworkEndian::read_u16(&[
                self.next_u8().unwrap(),
                self.next_u8().unwrap(),
            ]))
        }
    }

    /// Parses and returns the next four bytes as a [f32] if they are present, otherwaise returns [None].
    pub fn next_f32(&mut self) -> Option<f32> {
        if self.0.len() < 4 {
            None
        } else {
            Some(NetworkEndian::read_f32(&[
                self.next_u8().unwrap(),
                self.next_u8().unwrap(),
                self.next_u8().unwrap(),
                self.next_u8().unwrap(),
            ]))
        }
    }

    /// Creates a string from the next [size] bytes of the packet.
    ///
    /// Returns an Option containing the string or None if the [Packet] is not long enough.
    pub fn extract_string(&mut self, size: usize) -> Option<String> {
        if self.len() < 2 {
            None
        } else {
            let mut vec = Vec::new();
            for i in 0..size {
                vec.push(self.next_u8().unwrap());
            }
            Some(String::from_utf8_lossy(vec.as_ref()).to_string())
        }
    }
}

/// Extracts a string from a [VecDeque<u8>] where the first two bytes are a [u16] representing the length of the string.
/// That many bytes will be used to construct the string, being removed from the [VecDeque].
///
/// Returns an Option containing the string or None if the VecDeque was not long enough.
pub fn extract_string_u16_size(bytes: &mut VecDeque<u8>) -> Option<String> {
    if bytes.len() < 2 {
        None
    } else {
        let size =
            NetworkEndian::read_u16(&[bytes.pop_front().unwrap(), bytes.pop_front().unwrap()]);
        if bytes.len() < size as usize {
            None
        } else {
            let mut vec = Vec::new();
            for i in 0..size {
                vec.push(bytes.pop_front().unwrap());
            }
            Some(String::from_utf8_lossy(vec.as_ref()).to_string())
        }
    }
}

/// [PacketWriter] represents a new data packet and allows for adding components in order.
pub struct PacketWriter(Cursor<Vec<u8>>);

impl PacketWriter {
    /// Creates a new [PacketWriter]
    pub fn new() -> Self {
        PacketWriter(Cursor::new(Vec::new()))
    }

    /// Returns a [Vec<u8>] representing the packet.
    pub fn into_vec(self) -> Vec<u8> {
        self.0.into_inner()
    }

    /// Returns the length of the packet in bytes.
    pub fn len(&self) -> usize {
        self.0.get_ref().len()
    }

    /// Writes one [u8] to the end of the packet.
    pub fn write_u8(&mut self, val: u8) {
        self.0.write(&[val.to_be()]).unwrap();
    }

    /// Writes one [i8] to the end of the packet.
    pub fn write_i8(&mut self, val: i8) {
        self.0.write(&[val.to_be() as u8]).unwrap();
    }

    /// Writes one [u16] to the end of the packet.
    pub fn write_u16(&mut self, val: u16) {
        self.0.write_u16::<NetworkEndian>(val).unwrap();
    }

    /// Writes one [i16] to the end of the packet.
    pub fn write_i16(&mut self, val: i16) {
        self.0.write_i16::<NetworkEndian>(val).unwrap();
    }

    /// Writes one [u32] to the end of the packet.
    pub fn write_u32(&mut self, val: u32) {
        self.0.write_u32::<NetworkEndian>(val).unwrap();
    }

    /// Writes one [f32] to the end of the packet.
    pub fn write_f32(&mut self, val: f32) {
        self.0.write_f32::<NetworkEndian>(val).unwrap();
    }

    /// Writes a [String] to the end of the packet.
    pub fn write_string(&mut self, val: String) {
        self.0.write(val.as_bytes()).unwrap();
    }

    pub fn append_packet(&mut self, other: PacketWriter) {
        self.0.write(other.into_vec().as_ref()).unwrap();
    }

    pub fn write_slice(&mut self, slice: &[u8]) {
        self.0.write(slice).unwrap();
    }

    pub fn write_vec(&mut self, vec: Vec<u8>) {
        self.write_slice(vec.as_ref());
    }
}
