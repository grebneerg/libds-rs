use std::convert::From;

use packet::PacketReader;
use states::RobotMode;

pub struct Trace {
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

pub struct Status {
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

pub struct RioUdpPacket {
    pub sequence_num: u16,
    pub comm_version: u8,
    pub status: Status,
    pub trace: Trace,
    pub battery_voltage: f32,
    pub request_date: bool,
    pub tags: PacketReader, // TODO: parse tags from this Packet
}

impl RioUdpPacket {
    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() > 8 {
            None
        } else {
            let mut packet = PacketReader::from_vec(bytes);
            Some(RioUdpPacket {
                sequence_num: packet.next_u16().unwrap(),
                comm_version: packet.next_u8().unwrap(),
                status: packet.next_u8().unwrap().into(),
                trace: packet.next_u8().unwrap().into(),
                battery_voltage: f32::from(packet.next_u8().unwrap())
                    + f32::from(packet.next_u8().unwrap()) / 256.0,
                request_date: packet.next_u8().unwrap() == 0x01,
                tags: packet,
            })
        }
    }
}

pub enum RioTcpPacket {
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
    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() == 0 {
            None
        } else {
            use self::RioTcpPacket::*;
            let mut packet = PacketReader::from_vec(bytes);
            match packet.next_u8().unwrap() {
                0x00 => Some(RadioEvent({
                    let size = packet.len();
                    packet.extract_string(size).unwrap()
                })),
                0x04 => if packet.len() != 4 {
                    None
                } else {
                    Some(DisableFaults {
                        comms: packet.next_u16().unwrap(),
                        twelve_v: packet.next_u16().unwrap(),
                    })
                },
                0x05 => if packet.len() != 6 {
                    None
                } else {
                    Some(RailFaults {
                        six_v: packet.next_u16().unwrap(),
                        five_v: packet.next_u16().unwrap(),
                        three_point_three_v: packet.next_u16().unwrap(),
                    })
                },
                0x0a => if packet.len() < 5 {
                    None
                } else {
                    None // TODO
                },
                0x0b => if packet.len() < 16 {
                    None
                } else {
                    Some(ErrorMessage {
                        timestamp: packet.next_f32().unwrap(),
                        sequence_number: packet.next_u16().unwrap(),
                        print_msg: packet.next_u8().unwrap() == 0x01,
                        error_code: packet.next_u16().unwrap(),
                        is_error: packet.next_u8().unwrap() != 0,
                        details: {
                            let size = packet.next_u16().unwrap() as usize;
                            packet.extract_string(size).unwrap()
                        },
                        location: {
                            let size = packet.next_u16().unwrap() as usize;
                            packet.extract_string(size).unwrap()
                        },
                        call_stack: {
                            let size = packet.next_u16().unwrap() as usize;
                            packet.extract_string(size).unwrap()
                        },
                    })
                },
                0x0c => if packet.len() < 6 {
                    None
                } else {
                    Some(StandardOutput {
                        timestamp: packet.next_f32().unwrap(),
                        sequence_number: packet.next_u16().unwrap(),
                        message: {
                            let size = packet.len();
                            packet.extract_string(size).unwrap()
                        },
                    })
                },
                _ => None,
            }
        }
    }
}

pub enum DeviceType {
    Software = 0,
    CANTalon = 2,
    PDP = 8,
    PCM = 9,
}

pub enum RioPacket {
    Udp(RioUdpPacket),
    Tcp(RioTcpPacket),
}
