use bytes::PacketWriter;

#[derive(Clone)]
enum JoystickType {
    Unknown = -1,
}

#[derive(Clone)]
pub struct Joystick {
    stick_type: JoystickType,
    buttons: Vec<bool>,
    axes: Vec<i8>,
    povs: Vec<i16>,
}

impl Joystick {
    pub fn new(num_buttons: u8, num_axes: u8, num_povs: u8) -> Self {
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

    pub fn udp_tag(&self) -> Vec<u8> {
        let mut tag = PacketWriter::new();

        tag.write_u8(self.axes.len() as u8);
        for axis in &self.axes {
            tag.write_u8(*axis as u8); // this might work
        }

        tag.write_u8(self.buttons.len() as u8);
        let mut index = 7;
        let mut byte: u8 = 0;
        for button in &self.buttons {
            if *button {
                byte |= 1 << index;
            }
            index -= 1;
            if index < 0 {
                tag.write_u8(byte);
                byte = 0;
                index = 7;
            }
        }
        if index != 7 {
            tag.write_u8(byte);
        }

        tag.write_u8(self.povs.len() as u8);
        for pov in &self.povs {
            tag.write_u8(((pov >> 8) & 0xff as i16) as u8);
            tag.write_u8((pov & 0xff) as u8);
        }

        tag.into_vec()
    }
}
