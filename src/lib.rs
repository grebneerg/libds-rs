use std::default::Default;

mod joystick;
mod messages;
mod states;

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
}

impl DriverStation {
    pub fn new() -> Self {
        Default::default()
    }

    fn udp_packet(&mut self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();

        // Packet number in case they arrive out of order
        packet.push(((self.sequence_num >> 8) & 0xff) as u8);
        packet.push((self.sequence_num & 0xff) as u8);
        self.sequence_num += 1;

        packet.push(0x01); // comm version
        packet.push(self.control_byte()); // control byte
        packet.push(0); // TODO: actually restart code or rio with this byte.
        packet.push(self.alliance.to_position_u8()); // alliance

        // joystick tags
        for stick in &self.joysticks {
            if let Some(stick) = stick {
                let mut tag = stick.udp_tag();
                packet.push(tag.len() as u8 + 1); // size
                packet.push(0x0c); // id
                packet.extend(tag); // joystick tag info
            } else {
                // Empty joystick tag
                packet.push(0x01); // size
                packet.push(0x0c); // id
            }
        }

        packet
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
        }
    }
}
