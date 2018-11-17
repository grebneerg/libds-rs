use byteorder::{ByteOrder, NetworkEndian};
use std::collections::VecDeque;

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
