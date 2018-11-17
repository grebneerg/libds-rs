use std::collections::VecDeque;
use std::convert::From;

use byteorder::{ByteOrder, NetworkEndian};

use bytes::extract_string_u16_size;
use states::{Alliance, RobotMode};

pub(crate) struct Trace {
    robot_code: bool,
    is_roborio: bool,
    // these modes don't seem to quite line up with what we send
    test_mode: bool,
    auto_mode: bool,
    teleop_code: bool,
    disabled: bool,
}

impl From<u8> for Trace {
    fn from(byte: u8) -> Self {
        Trace {
            robot_code: byte & 0b0010_0000 != 0,
            is_roborio: byte & 0b0001_0000 != 0,
            test_mode: byte & 0b0000_1000 != 0,
            auto_mode: byte & 0b0000_0100 != 0,
            teleop_code: byte & 0b0000_0010 != 0,
            disabled: byte & 0b0000_0001 != 0,
        }
    }
}

pub(crate) struct Status {
    e_stop: bool,
    brownout: bool,
    code_initializing: bool,
    enabled: bool,
    mode: Option<RobotMode>,
}

impl From<u8> for Status {
    fn from(byte: u8) -> Self {
        Status {
            e_stop: byte & 0b1000_0000 != 0,
            brownout: byte & 0b0001_0000 != 0,
            code_initializing: byte & 0b0000_1000 != 0,
            enabled: byte & 0b0000_0100 != 0,
            mode: RobotMode::from(byte | 0b0000_0011),
        }
    }
}

struct RioUdpPacket {
    sequence_num: u16,
    comm_version: u8,
    status: Status,
    trace: Trace,
    battery_voltage: f32,
    request_date: bool,
    tags: VecDeque<u8>, // TODO: parse tags from this vec
}

impl RioUdpPacket {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() > 8 {
            None
        } else {
            let mut bytes = VecDeque::from(bytes);
            Some(RioUdpPacket {
                sequence_num: NetworkEndian::read_u16(&[
                    bytes.pop_front().unwrap(),
                    bytes.pop_front().unwrap(),
                ]),
                comm_version: bytes.pop_front().unwrap(),
                status: bytes.pop_front().unwrap().into(),
                trace: bytes.pop_front().unwrap().into(),
                battery_voltage: f32::from(bytes.pop_front().unwrap())
                    + f32::from(bytes.pop_front().unwrap()) / 256.0,
                request_date: bytes.pop_front().unwrap() == 0x01,
                tags: bytes,
            })
        }
    }
}

enum RioTcpPacket {
    RadioEvent(String), // 0x00
    UsageReport {
        // Ignoring this for now, apparently just forwarded to fms anyway.
        // 0x01
        team_num: String,
        unknown: u8,
        entries: Vec<u8>,
    },
    DisableFaults {
        // 0x04
        comms: u16,
        twelve_v: u16,
    },
    RailFaults {
        // 0x05
        six_v: u16,
        five_v: u16,
        three_point_three_v: u16,
    },
    VersionInfo {
        // 0x0a
        device_type: DeviceType,
        unknown: u16,
        id: u8,
        name: String,
        version: String,
    },
    ErrorMessage {
        // 0x0b
        timestamp: f32,
        sequence_number: u16,
        print_msg: bool,
        error_code: u16,
        is_error: bool, // false => warning
        details: String,
        location: String,
        call_stack: String,
    },
    StandardOutput {
        // 0x0c
        timestamp: f32,
        sequence_number: u16,
        message: String,
    },
    Unknown(Vec<u8>), // 0x0d
}

impl RioTcpPacket {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() == 0 {
            None
        } else {
            use self::RioTcpPacket::*;
            let mut bytes = VecDeque::from(bytes);
            match bytes.pop_front().unwrap() {
                0x00 => Some(RadioEvent(
                    String::from_utf8_lossy(Vec::from(bytes).as_ref()).to_string(),
                )),
                0x04 => if bytes.len() != 4 {
                    None
                } else {
                    Some(DisableFaults {
                        comms: NetworkEndian::read_u16(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                        twelve_v: NetworkEndian::read_u16(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                    })
                },
                0x05 => if bytes.len() != 6 {
                    None
                } else {
                    Some(RailFaults {
                        six_v: NetworkEndian::read_u16(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                        five_v: NetworkEndian::read_u16(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                        three_point_three_v: NetworkEndian::read_u16(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                    })
                },
                0x0a => if bytes.len() < 5 {
                    None
                } else {
                    None // TODO
                },
                0x0b => if bytes.len() < 16 {
                    None
                } else {
                    Some(ErrorMessage {
                        timestamp: NetworkEndian::read_f32(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                        sequence_number: NetworkEndian::read_u16(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                        print_msg: bytes.pop_front().unwrap() == 0x01,
                        error_code: NetworkEndian::read_u16(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                        is_error: bytes.pop_front().unwrap() != 0,
                        details: extract_string_u16_size(&mut bytes).unwrap(),
                        location: extract_string_u16_size(&mut bytes).unwrap(),
                        call_stack: extract_string_u16_size(&mut bytes).unwrap(),
                    })
                },
                0x0c => if bytes.len() < 6 {
                    None
                } else {
                    Some(StandardOutput {
                        timestamp: NetworkEndian::read_f32(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                        sequence_number: NetworkEndian::read_u16(&[
                            bytes.pop_front().unwrap(),
                            bytes.pop_front().unwrap(),
                        ]),
                        message: String::from_utf8_lossy(Vec::from(bytes).as_ref()).to_string(),
                    })
                },
                _ => None,
            }
        }
    }
}

enum DeviceType {
    Software = 0,
    CANTalon = 2,
    PDP = 8,
    PCM = 9,
}
