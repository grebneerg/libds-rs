use byteorder::{ByteOrder, NetworkEndian};
use std::collections::VecDeque;

/// [Packet] represents a data packet and allows for extracting components in order.
pub struct Packet(VecDeque<u8>);

impl Packet {
    /// Creates a [Packet] from the given [Vec<u8>].
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Packet(VecDeque::from(vec))
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
