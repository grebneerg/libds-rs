use std::convert::From;

use packet::PacketReader;
use states::RobotMode;

bitflags! {
	pub struct Trace: u8 {
		const ROBOT_CODE = 0b0010_0000;
		const IS_ROBORIO = 0b0001_0000;
		const TEST_MODE = 0b0000_1000;
		const AUTO_MODE = 0b0000_0100;
		const TELEOP_CODE = 0b0000_0010;
		const DISABLED = 0b0000_0001;
	}
}

bitflags! {
	pub struct Status: u8 {
		const ESTOP = 0b1000_0000;
		const BROWNOUT = 0b0001_0000;
		const CODE_INITIALIZING = 0b0000_1000;
		const ENABLED = 0b0000_0100;

		const TELEOP = 0b00;
        const TEST = 0b01;
        const AUTO = 0b10;
	}
}

impl Status {
	pub fn robot_mode(&self) -> Option<RobotMode> {
		return if self.contains(Status::AUTO) {
			Some(RobotMode::Auto)
		} else if self.contains(Status::TELEOP) {
			Some(RobotMode::Teleop)
		} else if self.contains(Status::TEST) {
			Some(RobotMode::Test)
		} else {
			None
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
                status: Status::from_bits(packet.next_u8().unwrap()).unwrap(),
                trace: Trace::from_bits(packet.next_u8().unwrap()).unwrap(),
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
