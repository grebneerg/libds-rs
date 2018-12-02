use joystick::Joystick;
use messages::rio::*;
use packet::PacketWriter;
use states::{Alliance, RobotMode};

use chrono::prelude::*;

const TIMEZONE: &'static str = "UTC";

#[derive(Clone)]
pub struct DriverStationState {
    pub joysticks: Vec<Option<Joystick>>,
    pub estop: bool,
    pub enabled: bool,
    pub mode: RobotMode,
    pub alliance: Alliance,
    pub game_data: String,
    pub competition: String,
    sequence_num: u16,
    request_time: bool,
}

impl DriverStationState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn udp_packet(&mut self) -> Vec<u8> {
        let mut packet = PacketWriter::new();

        // Packet number in case they arrive out of order
        packet.write_u16(self.sequence_num);
        self.sequence_num += 1;

        packet.write_u8(0x01); // comm version
        packet.write_u8(self.control_byte().bits()); // control byte
        packet.write_u8(Request::empty().bits()); // TODO: actually restart code or rio with this byte.
        packet.write_u8(self.alliance.to_position_u8()); // alliance

        // joystick tags
        for stick in &self.joysticks {
            if let Some(stick) = stick {
                let mut tag = stick.udp_tag();
                packet.write_u8(tag.len() as u8 + 1); // size
                packet.write_u8(0x0c); // id
                packet.write_vec(tag); // joystick tag info
            } else {
                // Empty joystick tag
                packet.write_u8(0x01); // size
                packet.write_u8(0x0c); // id
            }
        }

        // datetime and timezone
        if self.request_time {
            // timezone
            packet.write_u8(TIMEZONE.len() as u8 + 1); // size
            packet.write_u8(0x10); // id
            packet.write_slice(TIMEZONE.as_bytes());

            // date and time
            packet.write_vec(date_packet());
        }

        packet.into_vec()
    }

    fn control_byte(&self) -> Control {
        let mut byte = Control::empty();
        if self.estop {
            byte |= Control::ESTOP;
        }
        // fms is never connected, but if it were that would go here
        if self.enabled {
            byte |= Control::ENABLED;
        }

        byte |= Control::from_bits(self.mode as u8).unwrap();

        byte
    }

    pub fn update_from_tcp(&mut self, packet: RioTcpPacket) {
        // TODO: implement
    }

    pub fn update_from_udp(&mut self, packet: RioUdpPacket) {
        self.request_time = packet.request_date;
        // TODO: Finish implementing this
    }
}

fn date_packet() -> Vec<u8> {
    let mut packet = PacketWriter::new();
    let now = Utc::now();
    packet.write_u8(11); // size
    packet.write_u8(0x0f); // id
    let nanos = now.nanosecond();
    let micros = nanos / 1000;
    packet.write_u32(micros);
    packet.write_u8(now.second() as u8);
    packet.write_u8(now.minute() as u8);
    packet.write_u8(now.hour() as u8);
    packet.write_u8(now.day() as u8); // should this be day0?
    packet.write_u8(now.month0() as u8);
    packet.write_u8((now.year() - 1900) as u8);
    packet.into_vec()
}

impl Default for DriverStationState {
    fn default() -> Self {
        DriverStationState {
            joysticks: vec![None; 6],
            estop: false,
            enabled: false,
            mode: RobotMode::Teleop,
            alliance: Alliance::Red(1),
            game_data: String::new(),
            competition: String::from("unknown"),
            sequence_num: 0,
            request_time: false,
        }
    }
}

bitflags! {
    pub struct Control: u8 {
        const ESTOP = 0b1000_0000;
        const FMS_CONNECTED = 0b0000_1000;
        const ENABLED = 0b0000_0100;

        const TELEOP = 0b00;
        const TEST = 0b01;
        const AUTO = 0b10;
    }
}

impl Control {
    pub fn robot_mode(&self) -> Option<RobotMode> {
        return if self.contains(Control::AUTO) {
            Some(RobotMode::Auto)
        } else if self.contains(Control::TELEOP) {
            Some(RobotMode::Teleop)
        } else if self.contains(Control::TEST) {
            Some(RobotMode::Test)
        } else {
            None
        };
    }
}

bitflags! {
    pub struct Request: u8 {
        const REBOOT_ROBORIO = 0b1000;
        const RESTART_ROBOT_CODE = 0b0100;
    }
}
