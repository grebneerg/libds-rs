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

impl Trace {
    pub(crate) fn from_byte(byte: u8) -> Self {
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

impl Status {
    pub(crate) fn from_byte(byte: u8) -> Self {
        Status {
            e_stop: byte & 0b1000_0000 != 0,
            brownout: byte & 0b0001_0000 != 0,
            code_initializing: byte & 0b0000_1000 != 0,
            enabled: byte & 0b0000_0100 != 0,
            mode: RobotMode::from(byte | 0b0000_0011),
        }
    }
}
