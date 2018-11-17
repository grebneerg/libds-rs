extern crate byteorder;
extern crate chrono;

use chrono::prelude::*;

use std::default::Default;

mod bytes;
mod joystick;
mod messages;
mod states;

use bytes::PacketWriter;
use joystick::Joystick;
use states::{Alliance, RobotMode};

const TIMEZONE: &'static str = "UTC";

pub struct DriverStation {
    joysticks: Vec<Option<Joystick>>,
    estop: bool,
    enabled: bool,
    mode: RobotMode,
    alliance: Alliance,
    game_data: String,
    competition: String,
    sequence_num: u16,
    request_time: bool,
}

impl DriverStation {
    pub fn new() -> Self {
        Default::default()
    }

    fn udp_packet(&mut self) -> Vec<u8> {
        let mut packet = PacketWriter::new();

        // Packet number in case they arrive out of order
        packet.write_u16(self.sequence_num);
        self.sequence_num += 1;

        packet.write_u8(0x01); // comm version
        packet.write_u8(self.control_byte()); // control byte
        packet.write_u8(0); // TODO: actually restart code or rio with this byte.
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

    fn control_byte(&self) -> u8 {
        let mut byte: u8 = 0;
        if self.estop {
            byte |= 0b1000_0000;
        }
        // fms is never connected, but if it were that would go here
        if self.enabled {
            byte |= 0b0000_0100;
        }

        byte |= self.mode as u8;

        byte
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

impl Default for DriverStation {
    fn default() -> Self {
        DriverStation {
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
