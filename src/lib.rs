#[derive(Clone)]
enum JoystickType {
    Unknown = -1,
}

#[derive(Clone)]
struct Joystick {
    stick_type: JoystickType,
    buttons: Vec<bool>,
    axes: Vec<i8>,
    povs: Vec<i16>,
}

impl Joystick {
    fn new(num_buttons: u8, num_axes: u8, num_povs: u8) -> Self {
        Joystick {
            stick_type: JoystickType::Unknown,
            buttons: vec![false; num_buttons as usize],
            axes: vec![0; num_axes as usize],
            povs: vec![-1; num_povs as usize],
        }
    }

    fn num_buttons(&self) -> u8 {
        self.buttons.len() as u8
    }

    fn num_axes(&self) -> u8 {
        self.axes.len() as u8
    }

    fn num_povs(&self) -> u8 {
        self.povs.len() as u8
    }

    fn set_button(&mut self, index: u8, pressed: bool) -> Result<(), ()> {
        if self.buttons.len() as u8 >= index {
            Err(())
        } else {
            self.buttons[index as usize] = pressed;
            Ok(())
        }
    }

    fn set_axis(&mut self, index: u8, value: i8) -> Result<(), ()> {
        if self.axes.len() as u8 >= index {
            Err(())
        } else {
            self.axes[index as usize] = value;
            Ok(())
        }
    }

    fn set_pov(&mut self, index: u8, value: i16) -> Result<(), ()> {
        if self.povs.len() as u8 >= index {
            Err(())
        } else {
            self.povs[index as usize] = value;
            Ok(())
        }
    }

    fn to_tag(&self) -> Vec<u8> {
        let mut tag: Vec<u8> = Vec::new();

        tag.push(self.axes.len() as u8);
        for axis in &self.axes {
            tag.push(*axis as u8); // this might work
        }

        tag.push(self.buttons.len() as u8);
        let mut index = 7;
        let mut byte: u8 = 0;
        for button in &self.buttons {
            if *button {
                byte |= 1 << index;
            }
            index -= 1;
            if index < 0 {
                tag.push(byte);
                byte = 0;
                index = 7;
            }
        }
        if index != 7 {
            tag.push(byte);
        }

        tag.push(self.povs.len() as u8);
        for pov in &self.povs {
            tag.push(((pov >> 8) & 0xff as i16) as u8);
            tag.push((pov & 0xff) as u8);
        }

        tag
    }
}

#[derive(Copy, Clone)]
enum RobotMode {
    Teleop = 0,
    Test = 1,
    Auto = 2,
}

enum Alliance {
    Red(u8),
    Blue(u8),
}

impl Alliance {
    // TODO check bounds on both methods

    fn to_position_u8(&self) -> u8 {
        match self {
            Alliance::Red(pos) => pos - 1,
            Alliance::Blue(pos) => pos + 2,
        }
    }

    fn from_position_u8(pos: u8) -> Self {
        if pos < 3 {
            Alliance::Red(pos + 1)
        } else {
            Alliance::Blue(pos % 3 + 1)
        }
    }
}

enum MatchType {
    None = 0,
    Practice = 1,
    Qualification = 2,
    Elimination = 3,
}

struct DriverStation {
    joysticks: Vec<Option<Joystick>>,
    estop: bool,
    enabled: bool,
    mode: RobotMode,
    alliance: Alliance,
    game_data: String,
    competition: String,
	sequence_num: u16
}

impl DriverStation {
    fn new() -> Self {
        DriverStation {
            joysticks: vec![None; 6],
            estop: false,
            enabled: false,
            mode: RobotMode::Teleop,
            alliance: Alliance::Red(1),
            game_data: String::from("rrr"),
            competition: String::from("unknown"),
			sequence_num: 0
        }
    }

	fn udp_packet(&self) -> Vec<u8> {
		let mut packet: Vec<u8> = Vec::new();

		packet.push(((self.sequence_num >> 8) & 0xff) as u8 );
		packet.push((self.sequence_num & 0xff) as u8);

		packet.push(0x01); // comm version
		packet.push(self.control_byte()); // control byte
		packet.push(0); // TODO: actually restart code or rio with this byte.
		packet.push(self.alliance.to_position_u8()); // alliance

		// TODO: joystick tags

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
